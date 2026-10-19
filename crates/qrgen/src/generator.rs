use serde::Deserialize;
use serde_json::Value;
use textplate::{
    image::{Rgba, RgbaImage},
    Assets, Color, Fonts, NodeKind, Options, Renderer, Report, Template, TemplateString, Vars,
};

use crate::{Error, QrElement, QrStyle, Result};

/// The image each template is drawn onto.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Canvas {
    pub width: u32,
    /// Defaults to `width`.
    pub height: Option<u32>,
    pub background: Color,
    /// Name of an asset to draw on instead of a blank canvas (its size wins over width/height).
    /// May contain `{{placeholders}}`, e.g. `"photos/{{2}}"`.
    pub base_image: Option<TemplateString>,
}

impl Default for Canvas {
    fn default() -> Self {
        Self {
            width: 1000,
            height: None,
            background: Color::WHITE,
            base_image: None,
        }
    }
}

/// A rendered image and the layout warnings found while drawing it.
pub struct Output {
    pub image: RgbaImage,
    pub report: Report,
}

/// Renders a template with QR codes, once per set of variables.
///
/// `Generator` is `Send + Sync`: render rows in parallel from one instance.
pub struct Generator {
    renderer: Renderer,
    template: Template,
    canvas: Canvas,
}

impl Generator {
    /// `qr` sets the defaults for `qr` nodes. A `qr` node without `data` uses `{{data}}`.
    pub fn new(fonts: Fonts, mut template: Template, canvas: Canvas, qr: QrStyle) -> Self {
        default_qr_data(&mut template.nodes);
        let mut renderer = Renderer::new(fonts);
        renderer.register("qr", QrElement { defaults: qr });
        Self {
            renderer,
            template,
            canvas,
        }
    }

    pub fn with_options(mut self, options: Options) -> Self {
        self.renderer = self.renderer.with_options(options);
        self
    }

    pub fn template(&self) -> &Template {
        &self.template
    }

    pub fn canvas(&self) -> &Canvas {
        &self.canvas
    }

    /// Asset names this render needs (base image and `image` nodes), so the caller can load them.
    pub fn asset_names(&self, vars: &dyn Vars) -> Result<Vec<String>> {
        let mut names = Vec::new();
        if let Some(base) = &self.canvas.base_image {
            names.push(render_var(base, vars, "canvas.base_image")?);
        }
        collect_image_srcs(&self.template.nodes, vars, "nodes", &mut names)?;
        names.sort();
        names.dedup();
        Ok(names)
    }

    pub fn render(&self, vars: &dyn Vars, assets: &Assets) -> Result<Output> {
        let mut image = match &self.canvas.base_image {
            Some(base) => {
                let name = render_var(base, vars, "canvas.base_image")?;
                assets
                    .get(&name)
                    .ok_or(Error::MissingBaseImage(name))?
                    .as_ref()
                    .clone()
            }
            None => RgbaImage::from_pixel(
                self.canvas.width,
                self.canvas.height.unwrap_or(self.canvas.width),
                Rgba(self.canvas.background.to_array()),
            ),
        };
        let report = self
            .renderer
            .render(&self.template, &mut image, vars, assets)?;
        Ok(Output { image, report })
    }
}

/// The template used when none is given: one QR code filling the canvas.
pub fn default_template() -> Template {
    Template::from_json5(r#"[{ type: "qr" }]"#).expect("valid default template")
}

fn render_var(s: &TemplateString, vars: &dyn Vars, path: &str) -> Result<String> {
    s.render(vars).map(|v| v.into_owned()).map_err(|name| {
        textplate::Error::MissingVar {
            path: path.to_string(),
            name,
        }
        .into()
    })
}

fn default_qr_data(nodes: &mut [textplate::Node]) {
    for node in nodes {
        match &mut node.kind {
            NodeKind::Custom(c) if c.kind == "qr" => {
                c.props
                    .entry("data")
                    .or_insert_with(|| Value::String("{{data}}".into()));
            }
            NodeKind::Box(b) => default_qr_data(&mut b.children),
            _ => {}
        }
    }
}

fn collect_image_srcs(
    nodes: &[textplate::Node],
    vars: &dyn Vars,
    path: &str,
    out: &mut Vec<String>,
) -> Result<()> {
    for (i, node) in nodes.iter().enumerate() {
        let path = format!("{path}[{i}]");
        match &node.kind {
            NodeKind::Image(img) => out.push(render_var(&img.src, vars, &path)?),
            NodeKind::Box(b) => {
                collect_image_srcs(&b.children, vars, &format!("{path}.children"), out)?
            }
            _ => {}
        }
    }
    Ok(())
}
