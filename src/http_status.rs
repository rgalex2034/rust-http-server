use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum HttpStatusCodeError {
    InvalidHttpStatus,
}

impl Error for HttpStatusCodeError {}

impl Display for HttpStatusCodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

macro_rules! status_codes {
    (
        $(($num:expr, $const:ident, $reason:expr),)+
    ) => {
        #[derive(Clone, Copy)]
        pub enum HttpStatus {
            $( $const, )+
        }

        impl From<HttpStatus> for u32 {
            fn from(value: HttpStatus) -> Self {
                match value {
                    $(
                        HttpStatus::$const => $num,
                    )+
                }
            }
        }

        impl From<HttpStatus> for &str {
            fn from(value: HttpStatus) -> Self {
                match value {
                    $(
                        HttpStatus::$const => $reason,
                    )+
                }
            }
        }

        impl TryFrom<u32> for HttpStatus {
            type Error = HttpStatusCodeError;
            fn try_from(value: u32) -> Result<Self, Self::Error> {
                match value {
                    $(
                        $num => Ok(HttpStatus::$const),
                    )+
                    _ => return Err(HttpStatusCodeError::InvalidHttpStatus)
                }
            }
        }
    };
}

#[rustfmt::skip]
status_codes!(
    (100, Continue, "Continue"),
    (101, SwitchingProtocols, "Switching Protocols"),
    (200, Ok, "Ok"),
    (201, Created, "Created"),
    (202, Accepted, "Accepted"),
    (203, NonAuthoritativeInformation, "Non Authoritative Information"),
    (204, NoContent, "No Content"),
    (205, ResetContent, "Reset Content"),
    (206, PartialContent, "Partial Content"),
    //3XX
    (300, MultipleChoices, "Multiple Choices"),
    (301, MovedPermanently, "Moved Permanently"),
    (302, Found, "Found"),
    (303, SeeOther, "See Other"),
    (304, NotModified, "Not Modified"),
    (305, UseProxy, "Use Proxy"),
    (306, _306Unused, "(Unused)"),
    (307, TemporaryRedirect, "Temporary Redirect"),
    (308, PermanentRedirect, "Permanent Redirect"),
    //4XX
    (400, BadRequest, "Bad Request"),
    (401, Unauthorized, "Unauthorized"),
    (402, PaymentRequired, "Payment Required"),
    (403, Forbidden, "Forbidden"),
    (404, NotFound, "Not Found"),
    (405, MethodNotAllowed, "Method Not Allowed"),
    (406, NotAcceptable, "Not Acceptable"),
    (407, ProxyAuthenticationRequired, "Proxy Authentication Required"),
    (408, RequestTimeout, "Request Timeout"),
    (409, Conflict, "Conflict"),
    (410, Gone, "Gone"),
    (411, LengthRequired, "Length Required"),
    (412, PreconditionFailed, "Precondition Failed"),
    (413, ContentTooLarge, "Content Too Large"),
    (414, UriTooLong, "URI Too Long"),
    (415, UnsupportedMediaType, "Unsupported Media Type"),
    (416, RangeNotSatisfiable, "Range Not Satisfiable"),
    (417, ExpectationFailed, "Expectation Failed"),
    (418, _418Unused, "I'm A Teapot"),
    (421, MisdirectedRequest, "Misdirected Request"),
    (422, UnprocessableContent, "Unprocessable Content"),
    (426, UpgradeRequired, "Upgrade Required"),
    //5XX
    (500, InternalServerError, "Internal Server Error"),
);

impl Display for HttpStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = *self;
        let status_code: u32 = status.into();
        let status_msg: &str = status.into();

        write!(f, "{} {}", status_code, status_msg)
    }
}
