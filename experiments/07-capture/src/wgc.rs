//! Windows Graphics Capture.
//!
//! The expected winner, and the experiment exists to confirm it rather than to
//! assume it. It captures a window through the compositor, so it sees content
//! drawn by a swap chain — which is every game, and exactly what `PrintWindow`
//! cannot see.
//!
//! Two properties matter beyond the frame rate.
//!
//! `IsCursorCaptureEnabled = false` removes the pointer from the frame, which
//! `FR-PERC-006` needs: the cursor in a captured frame is the agent's own
//! pointer, and a perception layer that treats it as an element will click on
//! itself.
//!
//! `IsBorderRequired = false` suppresses the yellow capture border. Whether it
//! can be set from an unpackaged application on this build is known-good-matrix
//! question 7, and the answer decides whether every captured frame carries a
//! border the perception layer has to crop.

use std::sync::mpsc::{Receiver, TryRecvError, channel};
use std::time::Instant;

use windows::Foundation::TypedEventHandler;
use windows::Graphics::Capture::{
    Direct3D11CaptureFrame, Direct3D11CaptureFramePool, GraphicsCaptureItem, GraphicsCaptureSession,
};
use windows::Graphics::DirectX::Direct3D11::IDirect3DDevice;
use windows::Graphics::DirectX::DirectXPixelFormat;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_HARDWARE;
use windows::Win32::Graphics::Direct3D11::{
    D3D11_CPU_ACCESS_READ, D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_MAP_READ,
    D3D11_MAPPED_SUBRESOURCE, D3D11_TEXTURE2D_DESC, D3D11_USAGE_STAGING, D3D11CreateDevice,
    ID3D11Device, ID3D11DeviceContext, ID3D11Texture2D,
};
use windows::Win32::Graphics::Dxgi::IDXGIDevice;
use windows::Win32::System::WinRT::Direct3D11::{
    CreateDirect3D11DeviceFromDXGIDevice, IDirect3DDxgiInterfaceAccess,
};
use windows::Win32::System::WinRT::Graphics::Capture::IGraphicsCaptureItemInterop;
use windows::core::{IInspectable, Interface};

use crate::measure::Frame;

/// A live capture of one window.
pub(crate) struct Wgc {
    device: ID3D11Device,
    context: ID3D11DeviceContext,
    session: GraphicsCaptureSession,
    pool: Direct3D11CaptureFramePool,
    frames: Receiver<Direct3D11CaptureFrame>,
    staging: Option<ID3D11Texture2D>,
    width: u32,
    height: u32,
}

/// What was asked for when the session started, so the report can say whether
/// it was honoured rather than whether it was requested.
pub(crate) struct Requested {
    pub(crate) cursor_disabled: Result<(), String>,
    pub(crate) border_disabled: Result<(), String>,
}

impl Wgc {
    /// Start capturing `window`.
    ///
    /// # Errors
    ///
    /// Returns the step that failed. The two option settings are reported
    /// rather than fatal: a build that refuses `IsBorderRequired = false` still
    /// captures, and the refusal is the finding.
    pub(crate) fn start(window: HWND) -> Result<(Self, Requested), String> {
        Self::start_with(window, false)
    }

    /// Start a capture that keeps the border, for the comparison below.
    ///
    /// # Errors
    ///
    /// As [`Self::start`].
    pub(crate) fn start_bordered(window: HWND) -> Result<(Self, Requested), String> {
        Self::start_with(window, true)
    }

    fn start_with(window: HWND, border: bool) -> Result<(Self, Requested), String> {
        let mut device: Option<ID3D11Device> = None;
        let mut context: Option<ID3D11DeviceContext> = None;
        // SAFETY: every out parameter is a valid Option<Interface> slot, which
        // is the shape this call expects.
        unsafe {
            D3D11CreateDevice(
                None,
                D3D_DRIVER_TYPE_HARDWARE,
                windows::Win32::Foundation::HMODULE::default(),
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                None,
                windows::Win32::Graphics::Direct3D11::D3D11_SDK_VERSION,
                Some(&raw mut device),
                None,
                Some(&raw mut context),
            )
        }
        .map_err(|error| format!("D3D11CreateDevice: {error}"))?;

        let device = device.ok_or_else(|| "no device was returned".to_owned())?;
        let context = context.ok_or_else(|| "no device context was returned".to_owned())?;

        let dxgi: IDXGIDevice = device
            .cast()
            .map_err(|error| format!("the device is not a DXGI device: {error}"))?;
        // SAFETY: `dxgi` is a live DXGI device obtained from the D3D11 device.
        let inspectable = unsafe { CreateDirect3D11DeviceFromDXGIDevice(&dxgi) }
            .map_err(|error| format!("CreateDirect3D11DeviceFromDXGIDevice: {error}"))?;
        let winrt_device: IDirect3DDevice = inspectable
            .cast()
            .map_err(|error| format!("the interop device is not an IDirect3DDevice: {error}"))?;

        // A window is turned into a capture item through an interop interface
        // rather than through a WinRT constructor, because the window handle is
        // a Win32 concept the projection does not carry.
        let interop: IGraphicsCaptureItemInterop =
            windows::core::factory::<GraphicsCaptureItem, IGraphicsCaptureItemInterop>()
                .map_err(|error| format!("capture item factory: {error}"))?;
        // SAFETY: `window` is a live window handle supplied by the caller.
        let item: GraphicsCaptureItem = unsafe { interop.CreateForWindow(window) }
            .map_err(|error| format!("CreateForWindow: {error}"))?;

        let size = item.Size().map_err(|error| format!("item size: {error}"))?;
        let width = u32::try_from(size.Width).map_err(|_| "negative width".to_owned())?;
        let height = u32::try_from(size.Height).map_err(|_| "negative height".to_owned())?;

        let pool = Direct3D11CaptureFramePool::CreateFreeThreaded(
            &winrt_device,
            DirectXPixelFormat::B8G8R8A8UIntNormalized,
            // Two buffers. One is not enough to decouple the producer from the
            // reader; more would hide latency the measurement is trying to see.
            2,
            size,
        )
        .map_err(|error| format!("frame pool: {error}"))?;

        let (sender, frames) = channel();
        pool.FrameArrived(
            &TypedEventHandler::<Direct3D11CaptureFramePool, IInspectable>::new(move |pool, _| {
                if let Some(pool) = pool.as_ref()
                    && let Ok(frame) = pool.TryGetNextFrame()
                {
                    // A closed receiver means the run has ended; dropping the
                    // frame is correct rather than an error.
                    let _ = sender.send(frame);
                }
                Ok(())
            }),
        )
        .map_err(|error| format!("FrameArrived: {error}"))?;

        let session = pool
            .CreateCaptureSession(&item)
            .map_err(|error| format!("capture session: {error}"))?;

        // Reported, not fatal. Whether these are honoured is the finding.
        let requested = Requested {
            cursor_disabled: session
                .SetIsCursorCaptureEnabled(false)
                .map_err(|error| error.to_string()),
            border_disabled: session
                .SetIsBorderRequired(border)
                .map_err(|error| error.to_string()),
        };

        session
            .StartCapture()
            .map_err(|error| format!("StartCapture: {error}"))?;

        Ok((
            Self {
                device,
                context,
                session,
                pool,
                frames,
                staging: None,
                width,
                height,
            },
            requested,
        ))
    }

    /// Take the next frame the compositor delivered, if one is waiting.
    ///
    /// Non-blocking on purpose. A route that has nothing to give is a fact the
    /// measurement wants, and a blocking read would turn it into a hang.
    ///
    /// # Errors
    ///
    /// Returns the step that failed, or `Ok(None)` when no frame is pending.
    pub(crate) fn next(&mut self) -> Result<Option<Frame>, String> {
        let started = Instant::now();
        let frame = match self.frames.try_recv() {
            Ok(frame) => frame,
            Err(TryRecvError::Empty) => return Ok(None),
            Err(TryRecvError::Disconnected) => {
                return Err("the frame pool stopped delivering".to_owned());
            }
        };

        let surface = frame
            .Surface()
            .map_err(|error| format!("frame surface: {error}"))?;
        let access: IDirect3DDxgiInterfaceAccess = surface
            .cast()
            .map_err(|error| format!("surface interop: {error}"))?;
        // SAFETY: the surface is alive for as long as `frame` is, which outlives
        // this call.
        let texture: ID3D11Texture2D = unsafe { access.GetInterface() }
            .map_err(|error| format!("surface texture: {error}"))?;

        let pixels = self.read_back(&texture)?;
        Ok(Some(Frame {
            pixels,
            width: self.width as usize,
            height: self.height as usize,
            elapsed: started.elapsed(),
        }))
    }

    /// Copy a texture on the graphics device into main memory.
    ///
    /// The staging texture is created once and reused. Creating one per frame
    /// is the classic way to measure an allocator instead of a capture path.
    fn read_back(&mut self, texture: &ID3D11Texture2D) -> Result<Vec<u8>, String> {
        if self.staging.is_none() {
            let mut description = D3D11_TEXTURE2D_DESC::default();
            // SAFETY: `description` is a valid writable descriptor.
            unsafe { texture.GetDesc(&raw mut description) };
            description.Usage = D3D11_USAGE_STAGING;
            description.BindFlags = 0;
            description.CPUAccessFlags = D3D11_CPU_ACCESS_READ.0 as u32;
            description.MiscFlags = 0;

            let mut staging: Option<ID3D11Texture2D> = None;
            // SAFETY: the descriptor is filled in and the out parameter is a
            // valid slot.
            unsafe {
                self.device
                    .CreateTexture2D(&raw const description, None, Some(&raw mut staging))
            }
            .map_err(|error| format!("staging texture: {error}"))?;
            self.staging = staging;
        }

        let staging = self
            .staging
            .as_ref()
            .ok_or_else(|| "no staging texture".to_owned())?;

        // SAFETY: both textures are live and share a description apart from
        // usage flags, which is what CopyResource requires.
        unsafe { self.context.CopyResource(staging, texture) };

        let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
        // SAFETY: `staging` was created with CPU read access, and it is
        // unmapped below on every path.
        unsafe {
            self.context
                .Map(staging, 0, D3D11_MAP_READ, 0, Some(&raw mut mapped))
        }
        .map_err(|error| format!("Map: {error}"))?;

        let width = self.width as usize;
        let height = self.height as usize;
        let mut pixels = Vec::with_capacity(width * height * 3);
        // SAFETY: the mapped pointer is valid for `RowPitch * height` bytes
        // until Unmap, and the loop stays inside that.
        unsafe {
            let base = mapped.pData.cast::<u8>();
            for row in 0..height {
                let start = base.add(row * mapped.RowPitch as usize);
                for column in 0..width {
                    let pixel = start.add(column * 4);
                    pixels.push(*pixel.add(2));
                    pixels.push(*pixel.add(1));
                    pixels.push(*pixel);
                }
            }
            self.context.Unmap(staging, 0);
        }
        Ok(pixels)
    }
}

impl Drop for Wgc {
    fn drop(&mut self) {
        // Closing in this order matters: a session still delivering into a
        // closed pool is the one way this leaves a thread running after the
        // run ends.
        let _ = self.session.Close();
        let _ = self.pool.Close();
    }
}
