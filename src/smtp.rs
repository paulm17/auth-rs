use axum::extract::State;
use mail_send::{mail_builder::MessageBuilder, SmtpClientBuilder};
use std::{path::PathBuf, sync::Arc};
use rand::{distributions::Alphanumeric, Rng};
use serde_json::Value;

use crate::{template::TemplateEngine, AppState};

#[derive(Debug)]
pub struct EmailBaseParams {
  pub from: String,
  pub from_name: String,
  pub to: String,
  pub subject: String,
}

#[derive(Debug)]
pub enum EmailParams {
  PasswordReset {
    base: EmailBaseParams,
    code: String,
  },
  MagicLink {
    base: EmailBaseParams,
    code: String,
    redirect_to: String,
  },
}

pub async fn send_email(
  email_params: EmailParams,
  State(data): State<Arc<AppState>>,
) -> Result<bool, Box<dyn std::error::Error>> {
  let (template_name, template_params, base) = match email_params {
    EmailParams::PasswordReset { base, code } => {
        let params = serde_json::json!({
          "ConfirmationURL": format!(
            "{}/verify_code", data.env.server_url
          ),
          "Code": code,
        });
        
        ("password_reset", params, base)
    },
    EmailParams::MagicLink { base, code, redirect_to } => {
        let params = serde_json::json!({
          "ConfirmationURL": format!(
            "{}/verify_magiclink_code", data.env.server_url
          ),
          "Code": code,
          "RedirectTo": redirect_to
        });
        
        ("magic_link", params, base)
    },
    // Future email types can be handled here
};

let (html, text) = generate_email(GenerateOptions {
    template_dir: std::env::current_dir()?.join("src").join("templates"),
    template_params,
    template_name: template_name.to_string(),
})?;

  // Build email message
  let message = MessageBuilder::new()
    .from((base.from_name, base.from))
    .to(vec![base.to])
    .subject(base.subject)
    .html_body(html)
    .text_body(text);

  // Send email
  let result = SmtpClientBuilder::new(data.env.mailer_server.clone(), data.env.mailer_port.clone())
    .connect_plain()
    .await?
    .send(message)
    .await;

  match result {
    Ok(_) => Ok(true),
    Err(_e) => Err("failed to send email".into())
  }
}

#[derive(Debug)]
pub struct GenerateOptions {
  pub template_dir: PathBuf,
  pub template_params: Value,
  pub template_name: String,
}

fn generate_email(options: GenerateOptions) 
  -> Result<(String, String), Box<dyn std::error::Error>> {
  let mut engine = TemplateEngine::new(options.template_dir)?;
    
  // Load the template and its partials
  engine.load_template(&options.template_name)?;

  // Render both HTML and text versions using the pre-generated parameters
  let rendered = engine.render(&options.template_name, &options.template_params);

  match rendered {
    Ok(rendered) => Ok((rendered.html, rendered.text)),
    Err(_) => Err("failed to send email".into())
  }
}

pub fn generate_random_string() -> String {
  let mut rng = rand::thread_rng();
  let random_string: String = (0..32)
      .map(|_| rng.sample(Alphanumeric) as char)
      .collect();

  random_string
}