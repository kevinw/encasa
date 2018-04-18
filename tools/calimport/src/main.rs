extern crate hyper;
extern crate hyper_rustls;
extern crate yup_oauth2 as oauth2;
extern crate google_calendar3 as calendar3;
use calendar3::Channel;
use calendar3::{Result, Error};
use std::default::Default;
use oauth2::{Authenticator, DefaultAuthenticatorDelegate, ApplicationSecret, MemoryStorage};
use calendar3::CalendarHub;

fn main() {
    // Get an ApplicationSecret instance by some means. It contains the `client_id` and 
    // `client_secret`, among other things.
    /*
{
    "installed":{
        "client_id":"25778406323-nkrm98avc0i6kg7uinbum618v7nafd2k.apps.googleusercontent.com",
        "project_id":"quick-guru-201418",
        "auth_uri":"https://accounts.google.com/o/oauth2/auth",
        "token_uri":"https://accounts.google.com/o/oauth2/token",
        "auth_provider_x509_cert_url":"https://www.googleapis.com/oauth2/v1/certs",
        "client_secret":"boxYPBHQdaYM__lvCggiUe9R",
        "redirect_uris":["urn:ietf:wg:oauth:2.0:oob","http://localhost"]
    }
}
    */
    let secret: ApplicationSecret = ApplicationSecret {
        client_id: "25778406323-nkrm98avc0i6kg7uinbum618v7nafd2k.apps.googleusercontent.com".to_string(),
        auth_provider_x509_cert_url: Some("https://www.googleapis.com/oauth2/v1/certs".to_string()),
        client_secret: "boxYPBHQdaYM__lvCggiUe9R".to_string(),
    };
        Default::default();
    // Instantiate the authenticator. It will choose a suitable authentication flow for you, 
    // unless you replace  `None` with the desired Flow.
    // Provide your own `AuthenticatorDelegate` to adjust the way it operates and get feedback about 
    // what's going on. You probably want to bring in your own `TokenStorage` to persist tokens and
    // retrieve them from storage.
    let auth = Authenticator::new(&secret, DefaultAuthenticatorDelegate,
                                  hyper::Client::with_connector(hyper::net::HttpsConnector::new(hyper_rustls::TlsClient::new())),
                                  <MemoryStorage as Default>::default(), None);
    let mut hub = CalendarHub::new(hyper::Client::with_connector(hyper::net::HttpsConnector::new(hyper_rustls::TlsClient::new())), auth);
    // As the method needs a request, you would usually fill it with the desired information
    // into the respective structure. Some of the parts shown here might not be applicable !
    // Values shown here are possibly random and not representative !
    let mut req = Channel::default();
     
    // You can configure optional parameters by calling the respective setters at will, and
    // execute the final call using `doit()`.
    // Values shown here are possibly random and not representative !
    let result = hub.events().watch(req, "calendarId")
                 .updated_min("ea")
                 .time_zone("no")
                 .time_min("justo")
                 .time_max("justo")
                 .sync_token("et")
                 .single_events(true)
                 .show_hidden_invitations(true)
                 .show_deleted(false)
                 .add_shared_extended_property("Lorem")
                 .q("et")
                 .add_private_extended_property("duo")
                 .page_token("aliquyam")
                 .order_by("sea")
                 .max_results(-55)
                 .max_attendees(-75)
                 .i_cal_uid("erat")
                 .always_include_email(false)
                 .doit();
     
    match result {
        Err(e) => match e {
            // The Error enum provides details about what exactly happened.
            // You can also just use its `Debug`, `Display` or `Error` traits
             Error::HttpError(_)
            |Error::MissingAPIKey
            |Error::MissingToken(_)
            |Error::Cancelled
            |Error::UploadSizeLimitExceeded(_, _)
            |Error::Failure(_)
            |Error::BadRequest(_)
            |Error::FieldClash(_)
            |Error::JsonDecodeError(_, _) => println!("{}", e),
        },
        Ok(res) => println!("Success: {:?}", res),
    }
}
