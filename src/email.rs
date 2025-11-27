use resend_rs::{Resend, types::CreateEmailBaseOptions};
use std::env;

pub struct EmailService {
    client: Resend,
    from_email: String,
}

impl EmailService {
    pub fn new() -> Self {
        let api_key = env::var("RESEND_API_KEY").expect("RESEND_API_KEY must be set");
        let from_email = env::var("RESEND_FROM_EMAIL").unwrap_or_else(|_| "Split Bills <billsplitter@ddoffy.org>".to_string());
        let client = Resend::new(&api_key);
        Self { client, from_email }
    }

    pub async fn send_email(
        &self,
        to: Vec<String>,
        subject: &str,
        html_body: &str,
        cc: Option<Vec<String>>,
        bcc: Option<Vec<String>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let to_refs: Vec<&str> = to.iter().map(|s| s.as_str()).collect();
        
        let mut email = CreateEmailBaseOptions::new(&self.from_email, to_refs, subject)
            .with_html(html_body);

        if let Some(cc_list) = &cc {
             for cc_email in cc_list {
                 email = email.with_cc(cc_email);
             }
        }

        if let Some(bcc_list) = &bcc {
             for bcc_email in bcc_list {
                 email = email.with_bcc(bcc_email);
             }
        }
        
        self.client.emails.send(email).await?;
        Ok(())
    }
}
