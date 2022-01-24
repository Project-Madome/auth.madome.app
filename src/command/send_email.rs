use aws_sdk_sesv2::{
    error::SendEmailError,
    model::{Destination, EmailContent, EmailTemplateContent, Template},
    SdkError,
};
use sai::{Component, ComponentLifecycle, Injected};

use crate::{config::Config, error::CommandError};

use super::r#trait::Command;

pub const AUTHCODE_TEMPLATE_NAME: &str = "authcode_template";

#[derive(Component)]
#[lifecycle]
pub struct SendEmail {
    #[injected]
    config: Injected<Config>,

    aws_ses: Option<aws_sdk_sesv2::Client>,
}

#[async_trait::async_trait]
impl ComponentLifecycle for SendEmail {
    async fn start(&mut self) {
        self.aws_ses
            .replace(aws_sdk_sesv2::Client::new(self.config.aws_config()));

        if self.has_template(AUTHCODE_TEMPLATE_NAME).await {
            self.aws_ses()
                .delete_email_template()
                .template_name(AUTHCODE_TEMPLATE_NAME)
                .send()
                .await
                .expect("delete email template");
        }

        const AUTHCODE_TEMPLATE: &str = r#"<!DOCTYPE html><html><head><title>Madome</title><meta charset=utf-8><meta name="description"content="MadomeAuthCode"><meta http-equiv="cache-control"content="no-cache"><meta name="viewport"content="width=device-width,user-scalable=no,initial-scale=1,maximum-scale=1"><linkh ref="https://fonts.googleapis.com/css?family=Exo:300,600"rel="stylesheet"></head><body><div id="container"><span id="server">MadomeAuthCode</span><hr><div id="text">{{authcode}}</div><br/><div id="smallText">or</div><br/><div id="openurl"><a href="madome:///auth?value={{authcode}}">OpeninMadome</a></div></div></body></html><style>a,a:visited{color:currentColor}*{font-family:Exo,'Noto Sans',Ubuntu,Roboto,sans-serif;font-weight:300}a{text-decoration:underline}hr{width:10%;border-style:solid;border-color:#000;border-width:.5px;margin:25px auto}#container{position:absolute;text-align:center;top:100px;margin:20px;left:0;right:0}#text{font-size:3rem;font-weight:600;color:#444}#smallText{font-size:0.8rem;font-weight:100;color:#333}#openurl{font-size:1rem;font-weight:400;color:#555}#server{font-size:0.9rem;color:#666}</style>"#;

        self.aws_ses()
            .create_email_template()
            .template_name(AUTHCODE_TEMPLATE_NAME)
            .template_content(
                EmailTemplateContent::builder()
                    .subject("Authcode of madome.app")
                    .html(AUTHCODE_TEMPLATE)
                    .text("{{authcode}}")
                    .build(),
            )
            .send()
            .await
            .expect("create email template");
    }
}

impl SendEmail {
    async fn has_template(&self, template_name: &str) -> bool {
        let has_template = self
            .aws_ses()
            .get_email_template()
            .template_name(template_name)
            .send()
            .await
            .is_ok();

        has_template
    }

    fn authcode_template(content: &str) -> EmailContent {
        EmailContent::builder()
            .template(
                Template::builder()
                    .template_name(AUTHCODE_TEMPLATE_NAME)
                    .template_data(format!("{{\"authcode\": \"{}\"}}", content))
                    .build(),
            )
            .build()
    }

    fn aws_ses(&self) -> &aws_sdk_sesv2::Client {
        self.aws_ses.as_ref().unwrap()
    }
}

#[async_trait::async_trait]
impl Command<(String, String), ()> for SendEmail {
    type Error = crate::Error;

    async fn execute(&self, (email, content): (String, String)) -> Result<(), Self::Error> {
        let _output = self
            .aws_ses()
            .send_email()
            .from_email_address("verify@madome.app")
            .destination(Destination::builder().to_addresses(email).build())
            .content(Self::authcode_template(&content))
            .send()
            .await
            .map_err(Error::AwsSes)?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    AwsSes(#[from] SdkError<SendEmailError>),
}

impl From<Error> for crate::Error {
    fn from(err: Error) -> Self {
        CommandError::from(err).into()
    }
}

pub mod r#trait {
    use crate::command::r#trait::Command;

    pub trait SendEmail: Command<(String, String), (), Error = crate::Error> {}
}

#[cfg(test)]
pub mod tests {
    use sai::Component;

    use crate::command::r#trait::Command;

    use super::r#trait;

    #[derive(Component)]
    pub struct SendEmail;

    impl r#trait::SendEmail for SendEmail {}

    #[async_trait::async_trait]
    impl Command<(String, String), ()> for SendEmail {
        type Error = crate::Error;

        async fn execute(&self, _: (String, String)) -> Result<(), Self::Error> {
            Ok(())
        }
    }
}
