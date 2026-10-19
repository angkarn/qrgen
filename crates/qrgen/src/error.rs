/// Errors returned by qrgen.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Template, font, variable or layout error from textplate.
    #[error(transparent)]
    Render(#[from] textplate::Error),

    /// The data does not fit in a QR code at this error correction level.
    #[error("QR data is too long ({len} bytes) for error correction level {ecc}")]
    DataTooLong { len: usize, ecc: crate::Ecc },

    /// The QR box is smaller than one pixel per module.
    #[error("QR needs at least {needed}x{needed}px but its box is {width:.0}x{height:.0}px")]
    QrTooSmall {
        needed: u32,
        width: f32,
        height: f32,
    },

    /// The base image named by `canvas.base_image` was not supplied in the assets.
    #[error("missing base image {0:?}")]
    MissingBaseImage(String),

    #[error("invalid value: {0}")]
    Invalid(String),

    /// The job is invalid, or a file it names cannot be read.
    #[error("{0}")]
    Job(String),

    /// A row cannot be rendered: missing column or image, unusable output name, ...
    #[error("{0}")]
    Data(String),

    #[error("cannot encode PNG: {0}")]
    Png(#[from] image::ImageError),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
