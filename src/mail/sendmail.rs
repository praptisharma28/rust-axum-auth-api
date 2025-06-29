use std::{env, fs};
use lettre::{
    message::{header, SinglePart},
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport,
    Transport,
};

pub async fn send_email(
    to_email: &str,
    subject: &str,
    template_path: &str,
    placeholders: &[(String, String)]
) -> Result<(), Box<dyn std::error::Error>> {
    // Get all SMTP configuration
    let smtp_username = env::var("SMTP_USERNAME").unwrap_or_default();
    let smtp_password = env::var("SMTP_PASSWORD").unwrap_or_default();
    let smtp_server = env::var("SMTP_SERVER")?;
    let smtp_port: u16 = env::var("SMTP_PORT")?.parse()?;
    let smtp_from = env::var("SMTP_FROM_ADDRESS")?;

    // Read and process email template
    let mut html_template = fs::read_to_string(template_path)?;
    for (key, value) in placeholders {
        html_template = html_template.replace(key, value);
    }

    // Build email message
    let email = Message::builder()
        .from(smtp_from.parse()?) // Use SMTP_FROM_ADDRESS instead of SMTP_USERNAME
        .to(to_email.parse()?)
        .subject(subject)
        .header(header::ContentType::TEXT_HTML)
        .singlepart(SinglePart::builder()
            .header(header::ContentType::TEXT_HTML)
            .body(html_template)
        )?;

    // Build SMTP transport based on whether credentials are provided
    let mailer = if smtp_username.is_empty() || smtp_password.is_empty() {
        // No authentication (for MailHog)
        println!("🔧 Using SMTP without authentication (MailHog mode)");
        SmtpTransport::builder_dangerous(&smtp_server)
            .port(smtp_port)
            .build()
    } else {
        // With authentication (for production SMTP servers)
        println!("🔧 Using SMTP with authentication");
        let creds = Credentials::new(smtp_username, smtp_password);
        SmtpTransport::starttls_relay(&smtp_server)?
            .credentials(creds)
            .port(smtp_port)
            .build()
    };
    
    // Send email
    let result = mailer.send(&email);

    match result {
        Ok(_) => {
            println!("✅ Email sent successfully to: {}", to_email);
            Ok(())
        },
        Err(e) => {
            eprintln!("❌ Failed to send email: {:?}", e);
            eprintln!("📧 SMTP Config: {}:{}", smtp_server, smtp_port);
            eprintln!("📧 From: {}", smtp_from);
            eprintln!("📧 To: {}", to_email);
            Err(Box::new(e))
        }
    }
}