use handlebars::Handlebars;
use std::path::PathBuf;
use std::fs;

#[derive(Debug)]
pub struct EmailTemplate {
  pub html: String,
  pub text: String,
}

pub struct TemplateEngine {
  template_dir: PathBuf,
  handlebars: Handlebars<'static>,
}

impl TemplateEngine {
  pub fn new(template_dir: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
    let mut handlebars = Handlebars::new();
    // Enable directory support for partials
    handlebars.set_dev_mode(true);
    
    Ok(TemplateEngine {
      template_dir,
      handlebars,
    })
  }

  pub fn load_template(&mut self, template_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let html_path = self.template_dir.join(template_name).join("_.html.hbs");
    let text_path = self.template_dir.join(template_name).join("_.txt.hbs");

    // Register HTML and text templates with different names
    self.handlebars.register_template_file(
      &format!("{}_html", template_name),
      &html_path
    )?;
    self.handlebars.register_template_file(
      &format!("{}_text", template_name),
      &text_path
    )?;

    // Register partials
    let partials_dir = self.template_dir.join("partials");
    if partials_dir.exists() {
      for entry in fs::read_dir(partials_dir)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(name) = path.file_stem() {
          // Register as partial instead of regular template
          self.handlebars.register_partial(
              name.to_str().unwrap(),
              fs::read_to_string(path.clone())?.as_str()
          )?;
        }
      }
    }

    // Register layouts
    let layouts_dir = self.template_dir.join("layouts");
    if layouts_dir.exists() {
      for entry in fs::read_dir(layouts_dir)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(name) = path.file_stem() {
          // Register layouts as partials
          self.handlebars.register_partial(
              name.to_str().unwrap(),
              fs::read_to_string(path.clone())?.as_str()
          )?;
        }
      }
    }

    Ok(())
  }

  pub fn render(&self, template_name: &str, params: &serde_json::Value) -> Result<EmailTemplate, Box<dyn std::error::Error>> {
    let html = self.handlebars.render(
      &format!("{}_html", template_name),
      params
    )?;

    let text = self.handlebars.render(
      &format!("{}_text", template_name),
      params
    )?;

    Ok(EmailTemplate { html, text })
  }
}