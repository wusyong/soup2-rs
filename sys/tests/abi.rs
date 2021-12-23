// Generated by gir (https://github.com/gtk-rs/gir @ 05fe12c0b7e7)
// from ../gir-files (@ 7182204ef108)
// DO NOT EDIT

use soup_sys::*;
use std::mem::{align_of, size_of};
use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::path::Path;
use std::process::Command;
use std::str;
use tempfile::Builder;

static PACKAGES: &[&str] = &["libsoup-2.4"];

#[derive(Clone, Debug)]
struct Compiler {
    pub args: Vec<String>,
}

impl Compiler {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let mut args = get_var("CC", "cc")?;
        args.push("-Wno-deprecated-declarations".to_owned());
        // For _Generic
        args.push("-std=c11".to_owned());
        // For %z support in printf when using MinGW.
        args.push("-D__USE_MINGW_ANSI_STDIO".to_owned());
        args.extend(get_var("CFLAGS", "")?);
        args.extend(get_var("CPPFLAGS", "")?);
        args.extend(pkg_config_cflags(PACKAGES)?);
        Ok(Self { args })
    }

    pub fn compile(&self, src: &Path, out: &Path) -> Result<(), Box<dyn Error>> {
        let mut cmd = self.to_command();
        cmd.arg(src);
        cmd.arg("-o");
        cmd.arg(out);
        let status = cmd.spawn()?.wait()?;
        if !status.success() {
            return Err(format!("compilation command {:?} failed, {}", &cmd, status).into());
        }
        Ok(())
    }

    fn to_command(&self) -> Command {
        let mut cmd = Command::new(&self.args[0]);
        cmd.args(&self.args[1..]);
        cmd
    }
}

fn get_var(name: &str, default: &str) -> Result<Vec<String>, Box<dyn Error>> {
    match env::var(name) {
        Ok(value) => Ok(shell_words::split(&value)?),
        Err(env::VarError::NotPresent) => Ok(shell_words::split(default)?),
        Err(err) => Err(format!("{} {}", name, err).into()),
    }
}

fn pkg_config_cflags(packages: &[&str]) -> Result<Vec<String>, Box<dyn Error>> {
    if packages.is_empty() {
        return Ok(Vec::new());
    }
    let pkg_config = env::var_os("PKG_CONFIG")
        .unwrap_or_else(|| OsString::from("pkg-config"));
    let mut cmd = Command::new(pkg_config);
    cmd.arg("--cflags");
    cmd.args(packages);
    let out = cmd.output()?;
    if !out.status.success() {
        return Err(format!("command {:?} returned {}",
                           &cmd, out.status).into());
    }
    let stdout = str::from_utf8(&out.stdout)?;
    Ok(shell_words::split(stdout.trim())?)
}


#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct Layout {
    size: usize,
    alignment: usize,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
struct Results {
    /// Number of successfully completed tests.
    passed: usize,
    /// Total number of failed tests (including those that failed to compile).
    failed: usize,
}

impl Results {
    fn record_passed(&mut self) {
        self.passed += 1;
    }
    fn record_failed(&mut self) {
        self.failed += 1;
    }
    fn summary(&self) -> String {
        format!("{} passed; {} failed", self.passed, self.failed)
    }
    fn expect_total_success(&self) {
        if self.failed == 0 {
            println!("OK: {}", self.summary());
        } else {
            panic!("FAILED: {}", self.summary());
        };
    }
}

#[test]
fn cross_validate_constants_with_c() {
    let mut c_constants: Vec<(String, String)> = Vec::new();

    for l in get_c_output("constant").unwrap().lines() {
        let mut words = l.trim().split(';');
        let name = words.next().expect("Failed to parse name").to_owned();
        let value = words
            .next()
            .and_then(|s| s.parse().ok())
            .expect("Failed to parse value");
        c_constants.push((name, value));
    }

    let mut results = Results::default();

    for ((rust_name, rust_value), (c_name, c_value)) in
        RUST_CONSTANTS.iter().zip(c_constants.iter())
    {
        if rust_name != c_name {
            results.record_failed();
            eprintln!("Name mismatch:\nRust: {:?}\nC:    {:?}", rust_name, c_name,);
            continue;
        }

        if rust_value != c_value {
            results.record_failed();
            eprintln!(
                "Constant value mismatch for {}\nRust: {:?}\nC:    {:?}",
                rust_name, rust_value, &c_value
            );
            continue;
        }

        results.record_passed();
    }

    results.expect_total_success();
}

#[test]
fn cross_validate_layout_with_c() {
    let mut c_layouts = Vec::new();

    for l in get_c_output("layout").unwrap().lines() {
        let mut words = l.trim().split(';');
        let name = words.next().expect("Failed to parse name").to_owned();
        let size = words
            .next()
            .and_then(|s| s.parse().ok())
            .expect("Failed to parse size");
        let alignment = words
            .next()
            .and_then(|s| s.parse().ok())
            .expect("Failed to parse alignment");
        c_layouts.push((name, Layout { size, alignment }));
    }

    let mut results = Results::default();

    for ((rust_name, rust_layout), (c_name, c_layout)) in
        RUST_LAYOUTS.iter().zip(c_layouts.iter())
    {
        if rust_name != c_name {
            results.record_failed();
            eprintln!("Name mismatch:\nRust: {:?}\nC:    {:?}", rust_name, c_name,);
            continue;
        }

        if rust_layout != c_layout {
            results.record_failed();
            eprintln!(
                "Layout mismatch for {}\nRust: {:?}\nC:    {:?}",
                rust_name, rust_layout, &c_layout
            );
            continue;
        }

        results.record_passed();
    }

    results.expect_total_success();
}

fn get_c_output(name: &str) -> Result<String, Box<dyn Error>> {
    let tmpdir = Builder::new().prefix("abi").tempdir()?;
    let exe = tmpdir.path().join(name);
    let c_file = Path::new("tests").join(name).with_extension("c");

    let cc = Compiler::new().expect("configured compiler");
    cc.compile(&c_file, &exe)?;

    let mut abi_cmd = Command::new(exe);
    let output = abi_cmd.output()?;
    if !output.status.success() {
        return Err(format!("command {:?} failed, {:?}", &abi_cmd, &output).into());
    }

    Ok(String::from_utf8(output.stdout)?)
}

const RUST_LAYOUTS: &[(&str, Layout)] = &[
    ("SoupAddress", Layout {size: size_of::<SoupAddress>(), alignment: align_of::<SoupAddress>()}),
    ("SoupAddressClass", Layout {size: size_of::<SoupAddressClass>(), alignment: align_of::<SoupAddressClass>()}),
    ("SoupAddressFamily", Layout {size: size_of::<SoupAddressFamily>(), alignment: align_of::<SoupAddressFamily>()}),
    ("SoupAuth", Layout {size: size_of::<SoupAuth>(), alignment: align_of::<SoupAuth>()}),
    ("SoupAuthClass", Layout {size: size_of::<SoupAuthClass>(), alignment: align_of::<SoupAuthClass>()}),
    ("SoupAuthDomain", Layout {size: size_of::<SoupAuthDomain>(), alignment: align_of::<SoupAuthDomain>()}),
    ("SoupAuthDomainBasic", Layout {size: size_of::<SoupAuthDomainBasic>(), alignment: align_of::<SoupAuthDomainBasic>()}),
    ("SoupAuthDomainBasicClass", Layout {size: size_of::<SoupAuthDomainBasicClass>(), alignment: align_of::<SoupAuthDomainBasicClass>()}),
    ("SoupAuthDomainClass", Layout {size: size_of::<SoupAuthDomainClass>(), alignment: align_of::<SoupAuthDomainClass>()}),
    ("SoupAuthDomainDigest", Layout {size: size_of::<SoupAuthDomainDigest>(), alignment: align_of::<SoupAuthDomainDigest>()}),
    ("SoupAuthDomainDigestClass", Layout {size: size_of::<SoupAuthDomainDigestClass>(), alignment: align_of::<SoupAuthDomainDigestClass>()}),
    ("SoupAuthManager", Layout {size: size_of::<SoupAuthManager>(), alignment: align_of::<SoupAuthManager>()}),
    ("SoupAuthManagerClass", Layout {size: size_of::<SoupAuthManagerClass>(), alignment: align_of::<SoupAuthManagerClass>()}),
    ("SoupBuffer", Layout {size: size_of::<SoupBuffer>(), alignment: align_of::<SoupBuffer>()}),
    ("SoupCache", Layout {size: size_of::<SoupCache>(), alignment: align_of::<SoupCache>()}),
    ("SoupCacheClass", Layout {size: size_of::<SoupCacheClass>(), alignment: align_of::<SoupCacheClass>()}),
    ("SoupCacheResponse", Layout {size: size_of::<SoupCacheResponse>(), alignment: align_of::<SoupCacheResponse>()}),
    ("SoupCacheType", Layout {size: size_of::<SoupCacheType>(), alignment: align_of::<SoupCacheType>()}),
    ("SoupCacheability", Layout {size: size_of::<SoupCacheability>(), alignment: align_of::<SoupCacheability>()}),
    ("SoupConnectionState", Layout {size: size_of::<SoupConnectionState>(), alignment: align_of::<SoupConnectionState>()}),
    ("SoupContentDecoder", Layout {size: size_of::<SoupContentDecoder>(), alignment: align_of::<SoupContentDecoder>()}),
    ("SoupContentDecoderClass", Layout {size: size_of::<SoupContentDecoderClass>(), alignment: align_of::<SoupContentDecoderClass>()}),
    ("SoupContentSniffer", Layout {size: size_of::<SoupContentSniffer>(), alignment: align_of::<SoupContentSniffer>()}),
    ("SoupContentSnifferClass", Layout {size: size_of::<SoupContentSnifferClass>(), alignment: align_of::<SoupContentSnifferClass>()}),
    ("SoupCookie", Layout {size: size_of::<SoupCookie>(), alignment: align_of::<SoupCookie>()}),
    ("SoupCookieJar", Layout {size: size_of::<SoupCookieJar>(), alignment: align_of::<SoupCookieJar>()}),
    ("SoupCookieJarAcceptPolicy", Layout {size: size_of::<SoupCookieJarAcceptPolicy>(), alignment: align_of::<SoupCookieJarAcceptPolicy>()}),
    ("SoupCookieJarClass", Layout {size: size_of::<SoupCookieJarClass>(), alignment: align_of::<SoupCookieJarClass>()}),
    ("SoupCookieJarDB", Layout {size: size_of::<SoupCookieJarDB>(), alignment: align_of::<SoupCookieJarDB>()}),
    ("SoupCookieJarDBClass", Layout {size: size_of::<SoupCookieJarDBClass>(), alignment: align_of::<SoupCookieJarDBClass>()}),
    ("SoupCookieJarText", Layout {size: size_of::<SoupCookieJarText>(), alignment: align_of::<SoupCookieJarText>()}),
    ("SoupCookieJarTextClass", Layout {size: size_of::<SoupCookieJarTextClass>(), alignment: align_of::<SoupCookieJarTextClass>()}),
    ("SoupDate", Layout {size: size_of::<SoupDate>(), alignment: align_of::<SoupDate>()}),
    ("SoupDateFormat", Layout {size: size_of::<SoupDateFormat>(), alignment: align_of::<SoupDateFormat>()}),
    ("SoupEncoding", Layout {size: size_of::<SoupEncoding>(), alignment: align_of::<SoupEncoding>()}),
    ("SoupExpectation", Layout {size: size_of::<SoupExpectation>(), alignment: align_of::<SoupExpectation>()}),
    ("SoupHSTSEnforcer", Layout {size: size_of::<SoupHSTSEnforcer>(), alignment: align_of::<SoupHSTSEnforcer>()}),
    ("SoupHSTSEnforcerClass", Layout {size: size_of::<SoupHSTSEnforcerClass>(), alignment: align_of::<SoupHSTSEnforcerClass>()}),
    ("SoupHSTSEnforcerDB", Layout {size: size_of::<SoupHSTSEnforcerDB>(), alignment: align_of::<SoupHSTSEnforcerDB>()}),
    ("SoupHSTSEnforcerDBClass", Layout {size: size_of::<SoupHSTSEnforcerDBClass>(), alignment: align_of::<SoupHSTSEnforcerDBClass>()}),
    ("SoupHSTSPolicy", Layout {size: size_of::<SoupHSTSPolicy>(), alignment: align_of::<SoupHSTSPolicy>()}),
    ("SoupHTTPVersion", Layout {size: size_of::<SoupHTTPVersion>(), alignment: align_of::<SoupHTTPVersion>()}),
    ("SoupKnownStatusCode", Layout {size: size_of::<SoupKnownStatusCode>(), alignment: align_of::<SoupKnownStatusCode>()}),
    ("SoupLogger", Layout {size: size_of::<SoupLogger>(), alignment: align_of::<SoupLogger>()}),
    ("SoupLoggerClass", Layout {size: size_of::<SoupLoggerClass>(), alignment: align_of::<SoupLoggerClass>()}),
    ("SoupLoggerLogLevel", Layout {size: size_of::<SoupLoggerLogLevel>(), alignment: align_of::<SoupLoggerLogLevel>()}),
    ("SoupMemoryUse", Layout {size: size_of::<SoupMemoryUse>(), alignment: align_of::<SoupMemoryUse>()}),
    ("SoupMessage", Layout {size: size_of::<SoupMessage>(), alignment: align_of::<SoupMessage>()}),
    ("SoupMessageBody", Layout {size: size_of::<SoupMessageBody>(), alignment: align_of::<SoupMessageBody>()}),
    ("SoupMessageClass", Layout {size: size_of::<SoupMessageClass>(), alignment: align_of::<SoupMessageClass>()}),
    ("SoupMessageFlags", Layout {size: size_of::<SoupMessageFlags>(), alignment: align_of::<SoupMessageFlags>()}),
    ("SoupMessageHeadersIter", Layout {size: size_of::<SoupMessageHeadersIter>(), alignment: align_of::<SoupMessageHeadersIter>()}),
    ("SoupMessageHeadersType", Layout {size: size_of::<SoupMessageHeadersType>(), alignment: align_of::<SoupMessageHeadersType>()}),
    ("SoupMessagePriority", Layout {size: size_of::<SoupMessagePriority>(), alignment: align_of::<SoupMessagePriority>()}),
    ("SoupMultipartInputStream", Layout {size: size_of::<SoupMultipartInputStream>(), alignment: align_of::<SoupMultipartInputStream>()}),
    ("SoupMultipartInputStreamClass", Layout {size: size_of::<SoupMultipartInputStreamClass>(), alignment: align_of::<SoupMultipartInputStreamClass>()}),
    ("SoupPasswordManagerInterface", Layout {size: size_of::<SoupPasswordManagerInterface>(), alignment: align_of::<SoupPasswordManagerInterface>()}),
    ("SoupProxyResolverDefault", Layout {size: size_of::<SoupProxyResolverDefault>(), alignment: align_of::<SoupProxyResolverDefault>()}),
    ("SoupProxyResolverDefaultClass", Layout {size: size_of::<SoupProxyResolverDefaultClass>(), alignment: align_of::<SoupProxyResolverDefaultClass>()}),
    ("SoupProxyResolverInterface", Layout {size: size_of::<SoupProxyResolverInterface>(), alignment: align_of::<SoupProxyResolverInterface>()}),
    ("SoupProxyURIResolverInterface", Layout {size: size_of::<SoupProxyURIResolverInterface>(), alignment: align_of::<SoupProxyURIResolverInterface>()}),
    ("SoupRange", Layout {size: size_of::<SoupRange>(), alignment: align_of::<SoupRange>()}),
    ("SoupRequest", Layout {size: size_of::<SoupRequest>(), alignment: align_of::<SoupRequest>()}),
    ("SoupRequestClass", Layout {size: size_of::<SoupRequestClass>(), alignment: align_of::<SoupRequestClass>()}),
    ("SoupRequestData", Layout {size: size_of::<SoupRequestData>(), alignment: align_of::<SoupRequestData>()}),
    ("SoupRequestDataClass", Layout {size: size_of::<SoupRequestDataClass>(), alignment: align_of::<SoupRequestDataClass>()}),
    ("SoupRequestError", Layout {size: size_of::<SoupRequestError>(), alignment: align_of::<SoupRequestError>()}),
    ("SoupRequestFile", Layout {size: size_of::<SoupRequestFile>(), alignment: align_of::<SoupRequestFile>()}),
    ("SoupRequestFileClass", Layout {size: size_of::<SoupRequestFileClass>(), alignment: align_of::<SoupRequestFileClass>()}),
    ("SoupRequestHTTP", Layout {size: size_of::<SoupRequestHTTP>(), alignment: align_of::<SoupRequestHTTP>()}),
    ("SoupRequestHTTPClass", Layout {size: size_of::<SoupRequestHTTPClass>(), alignment: align_of::<SoupRequestHTTPClass>()}),
    ("SoupRequester", Layout {size: size_of::<SoupRequester>(), alignment: align_of::<SoupRequester>()}),
    ("SoupRequesterClass", Layout {size: size_of::<SoupRequesterClass>(), alignment: align_of::<SoupRequesterClass>()}),
    ("SoupRequesterError", Layout {size: size_of::<SoupRequesterError>(), alignment: align_of::<SoupRequesterError>()}),
    ("SoupSameSitePolicy", Layout {size: size_of::<SoupSameSitePolicy>(), alignment: align_of::<SoupSameSitePolicy>()}),
    ("SoupServer", Layout {size: size_of::<SoupServer>(), alignment: align_of::<SoupServer>()}),
    ("SoupServerClass", Layout {size: size_of::<SoupServerClass>(), alignment: align_of::<SoupServerClass>()}),
    ("SoupServerListenOptions", Layout {size: size_of::<SoupServerListenOptions>(), alignment: align_of::<SoupServerListenOptions>()}),
    ("SoupSession", Layout {size: size_of::<SoupSession>(), alignment: align_of::<SoupSession>()}),
    ("SoupSessionAsync", Layout {size: size_of::<SoupSessionAsync>(), alignment: align_of::<SoupSessionAsync>()}),
    ("SoupSessionAsyncClass", Layout {size: size_of::<SoupSessionAsyncClass>(), alignment: align_of::<SoupSessionAsyncClass>()}),
    ("SoupSessionClass", Layout {size: size_of::<SoupSessionClass>(), alignment: align_of::<SoupSessionClass>()}),
    ("SoupSessionFeatureInterface", Layout {size: size_of::<SoupSessionFeatureInterface>(), alignment: align_of::<SoupSessionFeatureInterface>()}),
    ("SoupSessionSync", Layout {size: size_of::<SoupSessionSync>(), alignment: align_of::<SoupSessionSync>()}),
    ("SoupSessionSyncClass", Layout {size: size_of::<SoupSessionSyncClass>(), alignment: align_of::<SoupSessionSyncClass>()}),
    ("SoupSocket", Layout {size: size_of::<SoupSocket>(), alignment: align_of::<SoupSocket>()}),
    ("SoupSocketClass", Layout {size: size_of::<SoupSocketClass>(), alignment: align_of::<SoupSocketClass>()}),
    ("SoupSocketIOStatus", Layout {size: size_of::<SoupSocketIOStatus>(), alignment: align_of::<SoupSocketIOStatus>()}),
    ("SoupStatus", Layout {size: size_of::<SoupStatus>(), alignment: align_of::<SoupStatus>()}),
    ("SoupTLDError", Layout {size: size_of::<SoupTLDError>(), alignment: align_of::<SoupTLDError>()}),
    ("SoupURI", Layout {size: size_of::<SoupURI>(), alignment: align_of::<SoupURI>()}),
    ("SoupWebsocketCloseCode", Layout {size: size_of::<SoupWebsocketCloseCode>(), alignment: align_of::<SoupWebsocketCloseCode>()}),
    ("SoupWebsocketConnection", Layout {size: size_of::<SoupWebsocketConnection>(), alignment: align_of::<SoupWebsocketConnection>()}),
    ("SoupWebsocketConnectionClass", Layout {size: size_of::<SoupWebsocketConnectionClass>(), alignment: align_of::<SoupWebsocketConnectionClass>()}),
    ("SoupWebsocketConnectionType", Layout {size: size_of::<SoupWebsocketConnectionType>(), alignment: align_of::<SoupWebsocketConnectionType>()}),
    ("SoupWebsocketDataType", Layout {size: size_of::<SoupWebsocketDataType>(), alignment: align_of::<SoupWebsocketDataType>()}),
    ("SoupWebsocketError", Layout {size: size_of::<SoupWebsocketError>(), alignment: align_of::<SoupWebsocketError>()}),
    ("SoupWebsocketExtension", Layout {size: size_of::<SoupWebsocketExtension>(), alignment: align_of::<SoupWebsocketExtension>()}),
    ("SoupWebsocketExtensionClass", Layout {size: size_of::<SoupWebsocketExtensionClass>(), alignment: align_of::<SoupWebsocketExtensionClass>()}),
    ("SoupWebsocketExtensionDeflate", Layout {size: size_of::<SoupWebsocketExtensionDeflate>(), alignment: align_of::<SoupWebsocketExtensionDeflate>()}),
    ("SoupWebsocketExtensionDeflateClass", Layout {size: size_of::<SoupWebsocketExtensionDeflateClass>(), alignment: align_of::<SoupWebsocketExtensionDeflateClass>()}),
    ("SoupWebsocketExtensionManager", Layout {size: size_of::<SoupWebsocketExtensionManager>(), alignment: align_of::<SoupWebsocketExtensionManager>()}),
    ("SoupWebsocketExtensionManagerClass", Layout {size: size_of::<SoupWebsocketExtensionManagerClass>(), alignment: align_of::<SoupWebsocketExtensionManagerClass>()}),
    ("SoupWebsocketState", Layout {size: size_of::<SoupWebsocketState>(), alignment: align_of::<SoupWebsocketState>()}),
    ("SoupXMLRPCError", Layout {size: size_of::<SoupXMLRPCError>(), alignment: align_of::<SoupXMLRPCError>()}),
    ("SoupXMLRPCFault", Layout {size: size_of::<SoupXMLRPCFault>(), alignment: align_of::<SoupXMLRPCFault>()}),
];

const RUST_CONSTANTS: &[(&str, &str)] = &[
    ("SOUP_ADDRESS_ANY_PORT", "0"),
    ("SOUP_ADDRESS_FAMILY", "family"),
    ("(gint) SOUP_ADDRESS_FAMILY_INVALID", "-1"),
    ("(gint) SOUP_ADDRESS_FAMILY_IPV4", "2"),
    ("(gint) SOUP_ADDRESS_FAMILY_IPV6", "10"),
    ("SOUP_ADDRESS_NAME", "name"),
    ("SOUP_ADDRESS_PHYSICAL", "physical"),
    ("SOUP_ADDRESS_PORT", "port"),
    ("SOUP_ADDRESS_PROTOCOL", "protocol"),
    ("SOUP_ADDRESS_SOCKADDR", "sockaddr"),
    ("SOUP_AUTH_DOMAIN_ADD_PATH", "add-path"),
    ("SOUP_AUTH_DOMAIN_BASIC_AUTH_CALLBACK", "auth-callback"),
    ("SOUP_AUTH_DOMAIN_BASIC_AUTH_DATA", "auth-data"),
    ("SOUP_AUTH_DOMAIN_DIGEST_AUTH_CALLBACK", "auth-callback"),
    ("SOUP_AUTH_DOMAIN_DIGEST_AUTH_DATA", "auth-data"),
    ("SOUP_AUTH_DOMAIN_FILTER", "filter"),
    ("SOUP_AUTH_DOMAIN_FILTER_DATA", "filter-data"),
    ("SOUP_AUTH_DOMAIN_GENERIC_AUTH_CALLBACK", "generic-auth-callback"),
    ("SOUP_AUTH_DOMAIN_GENERIC_AUTH_DATA", "generic-auth-data"),
    ("SOUP_AUTH_DOMAIN_PROXY", "proxy"),
    ("SOUP_AUTH_DOMAIN_REALM", "realm"),
    ("SOUP_AUTH_DOMAIN_REMOVE_PATH", "remove-path"),
    ("SOUP_AUTH_HOST", "host"),
    ("SOUP_AUTH_IS_AUTHENTICATED", "is-authenticated"),
    ("SOUP_AUTH_IS_FOR_PROXY", "is-for-proxy"),
    ("SOUP_AUTH_REALM", "realm"),
    ("SOUP_AUTH_SCHEME_NAME", "scheme-name"),
    ("(guint) SOUP_CACHE_CACHEABLE", "1"),
    ("(guint) SOUP_CACHE_INVALIDATES", "4"),
    ("(gint) SOUP_CACHE_RESPONSE_FRESH", "0"),
    ("(gint) SOUP_CACHE_RESPONSE_NEEDS_VALIDATION", "1"),
    ("(gint) SOUP_CACHE_RESPONSE_STALE", "2"),
    ("(gint) SOUP_CACHE_SHARED", "1"),
    ("(gint) SOUP_CACHE_SINGLE_USER", "0"),
    ("(guint) SOUP_CACHE_UNCACHEABLE", "2"),
    ("(guint) SOUP_CACHE_VALIDATES", "8"),
    ("SOUP_CHAR_HTTP_CTL", "16"),
    ("SOUP_CHAR_HTTP_SEPARATOR", "8"),
    ("SOUP_CHAR_URI_GEN_DELIMS", "2"),
    ("SOUP_CHAR_URI_PERCENT_ENCODED", "1"),
    ("SOUP_CHAR_URI_SUB_DELIMS", "4"),
    ("(gint) SOUP_CONNECTION_CONNECTING", "1"),
    ("(gint) SOUP_CONNECTION_DISCONNECTED", "5"),
    ("(gint) SOUP_CONNECTION_IDLE", "2"),
    ("(gint) SOUP_CONNECTION_IN_USE", "3"),
    ("(gint) SOUP_CONNECTION_NEW", "0"),
    ("(gint) SOUP_CONNECTION_REMOTE_DISCONNECTED", "4"),
    ("(gint) SOUP_COOKIE_JAR_ACCEPT_ALWAYS", "0"),
    ("(gint) SOUP_COOKIE_JAR_ACCEPT_GRANDFATHERED_THIRD_PARTY", "3"),
    ("(gint) SOUP_COOKIE_JAR_ACCEPT_NEVER", "1"),
    ("(gint) SOUP_COOKIE_JAR_ACCEPT_NO_THIRD_PARTY", "2"),
    ("SOUP_COOKIE_JAR_ACCEPT_POLICY", "accept-policy"),
    ("SOUP_COOKIE_JAR_DB_FILENAME", "filename"),
    ("SOUP_COOKIE_JAR_READ_ONLY", "read-only"),
    ("SOUP_COOKIE_JAR_TEXT_FILENAME", "filename"),
    ("SOUP_COOKIE_MAX_AGE_ONE_DAY", "0"),
    ("SOUP_COOKIE_MAX_AGE_ONE_HOUR", "3600"),
    ("SOUP_COOKIE_MAX_AGE_ONE_WEEK", "0"),
    ("SOUP_COOKIE_MAX_AGE_ONE_YEAR", "0"),
    ("(gint) SOUP_DATE_COOKIE", "2"),
    ("(gint) SOUP_DATE_HTTP", "1"),
    ("(gint) SOUP_DATE_ISO8601", "5"),
    ("(gint) SOUP_DATE_ISO8601_COMPACT", "4"),
    ("(gint) SOUP_DATE_ISO8601_FULL", "5"),
    ("(gint) SOUP_DATE_ISO8601_XMLRPC", "6"),
    ("(gint) SOUP_DATE_RFC2822", "3"),
    ("(gint) SOUP_ENCODING_BYTERANGES", "5"),
    ("(gint) SOUP_ENCODING_CHUNKED", "4"),
    ("(gint) SOUP_ENCODING_CONTENT_LENGTH", "2"),
    ("(gint) SOUP_ENCODING_EOF", "3"),
    ("(gint) SOUP_ENCODING_NONE", "1"),
    ("(gint) SOUP_ENCODING_UNRECOGNIZED", "0"),
    ("(guint) SOUP_EXPECTATION_CONTINUE", "2"),
    ("(guint) SOUP_EXPECTATION_UNRECOGNIZED", "1"),
    ("SOUP_FORM_MIME_TYPE_MULTIPART", "multipart/form-data"),
    ("SOUP_FORM_MIME_TYPE_URLENCODED", "application/x-www-form-urlencoded"),
    ("SOUP_HSTS_ENFORCER_DB_FILENAME", "filename"),
    ("SOUP_HSTS_POLICY_MAX_AGE_PAST", "0"),
    ("(gint) SOUP_HTTP_1_0", "0"),
    ("(gint) SOUP_HTTP_1_1", "1"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_ACCEPTED", "202"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_BAD_GATEWAY", "502"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_BAD_REQUEST", "400"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_CANCELLED", "1"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_CANT_CONNECT", "4"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_CANT_CONNECT_PROXY", "5"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_CANT_RESOLVE", "2"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_CANT_RESOLVE_PROXY", "3"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_CONFLICT", "409"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_CONTINUE", "100"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_CREATED", "201"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_EXPECTATION_FAILED", "417"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_FAILED_DEPENDENCY", "424"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_FORBIDDEN", "403"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_FOUND", "302"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_GATEWAY_TIMEOUT", "504"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_GONE", "410"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_HTTP_VERSION_NOT_SUPPORTED", "505"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_INSUFFICIENT_STORAGE", "507"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_INTERNAL_SERVER_ERROR", "500"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_INVALID_RANGE", "416"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_IO_ERROR", "7"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_LENGTH_REQUIRED", "411"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_LOCKED", "423"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_MALFORMED", "8"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_METHOD_NOT_ALLOWED", "405"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_MOVED_PERMANENTLY", "301"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_MOVED_TEMPORARILY", "302"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_MULTIPLE_CHOICES", "300"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_MULTI_STATUS", "207"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_NONE", "0"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_NON_AUTHORITATIVE", "203"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_NOT_ACCEPTABLE", "406"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_NOT_APPEARING_IN_THIS_PROTOCOL", "306"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_NOT_EXTENDED", "510"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_NOT_FOUND", "404"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_NOT_IMPLEMENTED", "501"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_NOT_MODIFIED", "304"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_NO_CONTENT", "204"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_OK", "200"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_PARTIAL_CONTENT", "206"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_PAYMENT_REQUIRED", "402"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_PRECONDITION_FAILED", "412"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_PROCESSING", "102"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_PROXY_AUTHENTICATION_REQUIRED", "407"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_PROXY_UNAUTHORIZED", "407"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_REQUESTED_RANGE_NOT_SATISFIABLE", "416"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_REQUEST_ENTITY_TOO_LARGE", "413"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_REQUEST_TIMEOUT", "408"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_REQUEST_URI_TOO_LONG", "414"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_RESET_CONTENT", "205"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_SEE_OTHER", "303"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_SERVICE_UNAVAILABLE", "503"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_SSL_FAILED", "6"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_SWITCHING_PROTOCOLS", "101"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_TEMPORARY_REDIRECT", "307"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_TLS_FAILED", "11"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_TOO_MANY_REDIRECTS", "10"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_TRY_AGAIN", "9"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_UNAUTHORIZED", "401"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_UNPROCESSABLE_ENTITY", "422"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_UNSUPPORTED_MEDIA_TYPE", "415"),
    ("(gint) SOUP_KNOWN_STATUS_CODE_USE_PROXY", "305"),
    ("SOUP_LOGGER_LEVEL", "level"),
    ("(gint) SOUP_LOGGER_LOG_BODY", "3"),
    ("(gint) SOUP_LOGGER_LOG_HEADERS", "2"),
    ("(gint) SOUP_LOGGER_LOG_MINIMAL", "1"),
    ("(gint) SOUP_LOGGER_LOG_NONE", "0"),
    ("SOUP_LOGGER_MAX_BODY_SIZE", "max-body-size"),
    ("SOUP_MAJOR_VERSION", "2"),
    ("(gint) SOUP_MEMORY_COPY", "2"),
    ("(gint) SOUP_MEMORY_STATIC", "0"),
    ("(gint) SOUP_MEMORY_TAKE", "1"),
    ("(gint) SOUP_MEMORY_TEMPORARY", "3"),
    ("(guint) SOUP_MESSAGE_CAN_REBUILD", "4"),
    ("(guint) SOUP_MESSAGE_CERTIFICATE_TRUSTED", "32"),
    ("(guint) SOUP_MESSAGE_CONTENT_DECODED", "16"),
    ("(guint) SOUP_MESSAGE_DO_NOT_USE_AUTH_CACHE", "512"),
    ("SOUP_MESSAGE_FIRST_PARTY", "first-party"),
    ("SOUP_MESSAGE_FLAGS", "flags"),
    ("(gint) SOUP_MESSAGE_HEADERS_MULTIPART", "2"),
    ("(gint) SOUP_MESSAGE_HEADERS_REQUEST", "0"),
    ("(gint) SOUP_MESSAGE_HEADERS_RESPONSE", "1"),
    ("SOUP_MESSAGE_HTTP_VERSION", "http-version"),
    ("(guint) SOUP_MESSAGE_IDEMPOTENT", "128"),
    ("(guint) SOUP_MESSAGE_IGNORE_CONNECTION_LIMITS", "256"),
    ("SOUP_MESSAGE_IS_TOP_LEVEL_NAVIGATION", "is-top-level-navigation"),
    ("SOUP_MESSAGE_METHOD", "method"),
    ("(guint) SOUP_MESSAGE_NEW_CONNECTION", "64"),
    ("(guint) SOUP_MESSAGE_NO_REDIRECT", "2"),
    ("(guint) SOUP_MESSAGE_OVERWRITE_CHUNKS", "8"),
    ("SOUP_MESSAGE_PRIORITY", "priority"),
    ("(gint) SOUP_MESSAGE_PRIORITY_HIGH", "3"),
    ("(gint) SOUP_MESSAGE_PRIORITY_LOW", "1"),
    ("(gint) SOUP_MESSAGE_PRIORITY_NORMAL", "2"),
    ("(gint) SOUP_MESSAGE_PRIORITY_VERY_HIGH", "4"),
    ("(gint) SOUP_MESSAGE_PRIORITY_VERY_LOW", "0"),
    ("SOUP_MESSAGE_REASON_PHRASE", "reason-phrase"),
    ("SOUP_MESSAGE_REQUEST_BODY", "request-body"),
    ("SOUP_MESSAGE_REQUEST_BODY_DATA", "request-body-data"),
    ("SOUP_MESSAGE_REQUEST_HEADERS", "request-headers"),
    ("SOUP_MESSAGE_RESPONSE_BODY", "response-body"),
    ("SOUP_MESSAGE_RESPONSE_BODY_DATA", "response-body-data"),
    ("SOUP_MESSAGE_RESPONSE_HEADERS", "response-headers"),
    ("SOUP_MESSAGE_SERVER_SIDE", "server-side"),
    ("SOUP_MESSAGE_SITE_FOR_COOKIES", "site-for-cookies"),
    ("SOUP_MESSAGE_STATUS_CODE", "status-code"),
    ("SOUP_MESSAGE_TLS_CERTIFICATE", "tls-certificate"),
    ("SOUP_MESSAGE_TLS_ERRORS", "tls-errors"),
    ("SOUP_MESSAGE_URI", "uri"),
    ("(gint) SOUP_REQUESTER_ERROR_BAD_URI", "0"),
    ("(gint) SOUP_REQUESTER_ERROR_UNSUPPORTED_URI_SCHEME", "1"),
    ("(gint) SOUP_REQUEST_ERROR_BAD_URI", "0"),
    ("(gint) SOUP_REQUEST_ERROR_ENCODING", "3"),
    ("(gint) SOUP_REQUEST_ERROR_PARSING", "2"),
    ("(gint) SOUP_REQUEST_ERROR_UNSUPPORTED_URI_SCHEME", "1"),
    ("SOUP_REQUEST_SESSION", "session"),
    ("SOUP_REQUEST_URI", "uri"),
    ("(gint) SOUP_SAME_SITE_POLICY_LAX", "1"),
    ("(gint) SOUP_SAME_SITE_POLICY_NONE", "0"),
    ("(gint) SOUP_SAME_SITE_POLICY_STRICT", "2"),
    ("SOUP_SERVER_ADD_WEBSOCKET_EXTENSION", "add-websocket-extension"),
    ("SOUP_SERVER_ASYNC_CONTEXT", "async-context"),
    ("SOUP_SERVER_HTTPS_ALIASES", "https-aliases"),
    ("SOUP_SERVER_HTTP_ALIASES", "http-aliases"),
    ("SOUP_SERVER_INTERFACE", "interface"),
    ("(guint) SOUP_SERVER_LISTEN_HTTPS", "1"),
    ("(guint) SOUP_SERVER_LISTEN_IPV4_ONLY", "2"),
    ("(guint) SOUP_SERVER_LISTEN_IPV6_ONLY", "4"),
    ("SOUP_SERVER_PORT", "port"),
    ("SOUP_SERVER_RAW_PATHS", "raw-paths"),
    ("SOUP_SERVER_REMOVE_WEBSOCKET_EXTENSION", "remove-websocket-extension"),
    ("SOUP_SERVER_SERVER_HEADER", "server-header"),
    ("SOUP_SERVER_SSL_CERT_FILE", "ssl-cert-file"),
    ("SOUP_SERVER_SSL_KEY_FILE", "ssl-key-file"),
    ("SOUP_SERVER_TLS_CERTIFICATE", "tls-certificate"),
    ("SOUP_SESSION_ACCEPT_LANGUAGE", "accept-language"),
    ("SOUP_SESSION_ACCEPT_LANGUAGE_AUTO", "accept-language-auto"),
    ("SOUP_SESSION_ADD_FEATURE", "add-feature"),
    ("SOUP_SESSION_ADD_FEATURE_BY_TYPE", "add-feature-by-type"),
    ("SOUP_SESSION_ASYNC_CONTEXT", "async-context"),
    ("SOUP_SESSION_HTTPS_ALIASES", "https-aliases"),
    ("SOUP_SESSION_HTTP_ALIASES", "http-aliases"),
    ("SOUP_SESSION_IDLE_TIMEOUT", "idle-timeout"),
    ("SOUP_SESSION_LOCAL_ADDRESS", "local-address"),
    ("SOUP_SESSION_MAX_CONNS", "max-conns"),
    ("SOUP_SESSION_MAX_CONNS_PER_HOST", "max-conns-per-host"),
    ("SOUP_SESSION_PROXY_RESOLVER", "proxy-resolver"),
    ("SOUP_SESSION_PROXY_URI", "proxy-uri"),
    ("SOUP_SESSION_REMOVE_FEATURE_BY_TYPE", "remove-feature-by-type"),
    ("SOUP_SESSION_SSL_CA_FILE", "ssl-ca-file"),
    ("SOUP_SESSION_SSL_STRICT", "ssl-strict"),
    ("SOUP_SESSION_SSL_USE_SYSTEM_CA_FILE", "ssl-use-system-ca-file"),
    ("SOUP_SESSION_TIMEOUT", "timeout"),
    ("SOUP_SESSION_TLS_DATABASE", "tls-database"),
    ("SOUP_SESSION_TLS_INTERACTION", "tls-interaction"),
    ("SOUP_SESSION_USER_AGENT", "user-agent"),
    ("SOUP_SESSION_USE_NTLM", "use-ntlm"),
    ("SOUP_SESSION_USE_THREAD_CONTEXT", "use-thread-context"),
    ("SOUP_SOCKET_ASYNC_CONTEXT", "async-context"),
    ("(gint) SOUP_SOCKET_EOF", "2"),
    ("(gint) SOUP_SOCKET_ERROR", "3"),
    ("SOUP_SOCKET_FLAG_NONBLOCKING", "non-blocking"),
    ("SOUP_SOCKET_IS_SERVER", "is-server"),
    ("SOUP_SOCKET_LOCAL_ADDRESS", "local-address"),
    ("(gint) SOUP_SOCKET_OK", "0"),
    ("SOUP_SOCKET_REMOTE_ADDRESS", "remote-address"),
    ("SOUP_SOCKET_SSL_CREDENTIALS", "ssl-creds"),
    ("SOUP_SOCKET_SSL_FALLBACK", "ssl-fallback"),
    ("SOUP_SOCKET_SSL_STRICT", "ssl-strict"),
    ("SOUP_SOCKET_TIMEOUT", "timeout"),
    ("SOUP_SOCKET_TLS_CERTIFICATE", "tls-certificate"),
    ("SOUP_SOCKET_TLS_ERRORS", "tls-errors"),
    ("SOUP_SOCKET_TRUSTED_CERTIFICATE", "trusted-certificate"),
    ("SOUP_SOCKET_USE_THREAD_CONTEXT", "use-thread-context"),
    ("(gint) SOUP_SOCKET_WOULD_BLOCK", "1"),
    ("(gint) SOUP_STATUS_ACCEPTED", "202"),
    ("(gint) SOUP_STATUS_BAD_GATEWAY", "502"),
    ("(gint) SOUP_STATUS_BAD_REQUEST", "400"),
    ("(gint) SOUP_STATUS_CANCELLED", "1"),
    ("(gint) SOUP_STATUS_CANT_CONNECT", "4"),
    ("(gint) SOUP_STATUS_CANT_CONNECT_PROXY", "5"),
    ("(gint) SOUP_STATUS_CANT_RESOLVE", "2"),
    ("(gint) SOUP_STATUS_CANT_RESOLVE_PROXY", "3"),
    ("(gint) SOUP_STATUS_CONFLICT", "409"),
    ("(gint) SOUP_STATUS_CONTINUE", "100"),
    ("(gint) SOUP_STATUS_CREATED", "201"),
    ("(gint) SOUP_STATUS_EXPECTATION_FAILED", "417"),
    ("(gint) SOUP_STATUS_FAILED_DEPENDENCY", "424"),
    ("(gint) SOUP_STATUS_FORBIDDEN", "403"),
    ("(gint) SOUP_STATUS_FOUND", "302"),
    ("(gint) SOUP_STATUS_GATEWAY_TIMEOUT", "504"),
    ("(gint) SOUP_STATUS_GONE", "410"),
    ("(gint) SOUP_STATUS_HTTP_VERSION_NOT_SUPPORTED", "505"),
    ("(gint) SOUP_STATUS_INSUFFICIENT_STORAGE", "507"),
    ("(gint) SOUP_STATUS_INTERNAL_SERVER_ERROR", "500"),
    ("(gint) SOUP_STATUS_INVALID_RANGE", "416"),
    ("(gint) SOUP_STATUS_IO_ERROR", "7"),
    ("(gint) SOUP_STATUS_LENGTH_REQUIRED", "411"),
    ("(gint) SOUP_STATUS_LOCKED", "423"),
    ("(gint) SOUP_STATUS_MALFORMED", "8"),
    ("(gint) SOUP_STATUS_METHOD_NOT_ALLOWED", "405"),
    ("(gint) SOUP_STATUS_MOVED_PERMANENTLY", "301"),
    ("(gint) SOUP_STATUS_MOVED_TEMPORARILY", "302"),
    ("(gint) SOUP_STATUS_MULTIPLE_CHOICES", "300"),
    ("(gint) SOUP_STATUS_MULTI_STATUS", "207"),
    ("(gint) SOUP_STATUS_NONE", "0"),
    ("(gint) SOUP_STATUS_NON_AUTHORITATIVE", "203"),
    ("(gint) SOUP_STATUS_NOT_ACCEPTABLE", "406"),
    ("(gint) SOUP_STATUS_NOT_APPEARING_IN_THIS_PROTOCOL", "306"),
    ("(gint) SOUP_STATUS_NOT_EXTENDED", "510"),
    ("(gint) SOUP_STATUS_NOT_FOUND", "404"),
    ("(gint) SOUP_STATUS_NOT_IMPLEMENTED", "501"),
    ("(gint) SOUP_STATUS_NOT_MODIFIED", "304"),
    ("(gint) SOUP_STATUS_NO_CONTENT", "204"),
    ("(gint) SOUP_STATUS_OK", "200"),
    ("(gint) SOUP_STATUS_PARTIAL_CONTENT", "206"),
    ("(gint) SOUP_STATUS_PAYMENT_REQUIRED", "402"),
    ("(gint) SOUP_STATUS_PERMANENT_REDIRECT", "308"),
    ("(gint) SOUP_STATUS_PRECONDITION_FAILED", "412"),
    ("(gint) SOUP_STATUS_PROCESSING", "102"),
    ("(gint) SOUP_STATUS_PROXY_AUTHENTICATION_REQUIRED", "407"),
    ("(gint) SOUP_STATUS_PROXY_UNAUTHORIZED", "407"),
    ("(gint) SOUP_STATUS_REQUESTED_RANGE_NOT_SATISFIABLE", "416"),
    ("(gint) SOUP_STATUS_REQUEST_ENTITY_TOO_LARGE", "413"),
    ("(gint) SOUP_STATUS_REQUEST_TIMEOUT", "408"),
    ("(gint) SOUP_STATUS_REQUEST_URI_TOO_LONG", "414"),
    ("(gint) SOUP_STATUS_RESET_CONTENT", "205"),
    ("(gint) SOUP_STATUS_SEE_OTHER", "303"),
    ("(gint) SOUP_STATUS_SERVICE_UNAVAILABLE", "503"),
    ("(gint) SOUP_STATUS_SSL_FAILED", "6"),
    ("(gint) SOUP_STATUS_SWITCHING_PROTOCOLS", "101"),
    ("(gint) SOUP_STATUS_TEMPORARY_REDIRECT", "307"),
    ("(gint) SOUP_STATUS_TLS_FAILED", "11"),
    ("(gint) SOUP_STATUS_TOO_MANY_REDIRECTS", "10"),
    ("(gint) SOUP_STATUS_TRY_AGAIN", "9"),
    ("(gint) SOUP_STATUS_UNAUTHORIZED", "401"),
    ("(gint) SOUP_STATUS_UNPROCESSABLE_ENTITY", "422"),
    ("(gint) SOUP_STATUS_UNSUPPORTED_MEDIA_TYPE", "415"),
    ("(gint) SOUP_STATUS_USE_PROXY", "305"),
    ("(gint) SOUP_TLD_ERROR_INVALID_HOSTNAME", "0"),
    ("(gint) SOUP_TLD_ERROR_IS_IP_ADDRESS", "1"),
    ("(gint) SOUP_TLD_ERROR_NOT_ENOUGH_DOMAINS", "2"),
    ("(gint) SOUP_TLD_ERROR_NO_BASE_DOMAIN", "3"),
    ("(gint) SOUP_TLD_ERROR_NO_PSL_DATA", "4"),
    ("(gint) SOUP_WEBSOCKET_CLOSE_ABNORMAL", "1006"),
    ("(gint) SOUP_WEBSOCKET_CLOSE_BAD_DATA", "1007"),
    ("(gint) SOUP_WEBSOCKET_CLOSE_GOING_AWAY", "1001"),
    ("(gint) SOUP_WEBSOCKET_CLOSE_NORMAL", "1000"),
    ("(gint) SOUP_WEBSOCKET_CLOSE_NO_EXTENSION", "1010"),
    ("(gint) SOUP_WEBSOCKET_CLOSE_NO_STATUS", "1005"),
    ("(gint) SOUP_WEBSOCKET_CLOSE_POLICY_VIOLATION", "1008"),
    ("(gint) SOUP_WEBSOCKET_CLOSE_PROTOCOL_ERROR", "1002"),
    ("(gint) SOUP_WEBSOCKET_CLOSE_SERVER_ERROR", "1011"),
    ("(gint) SOUP_WEBSOCKET_CLOSE_TLS_HANDSHAKE", "1015"),
    ("(gint) SOUP_WEBSOCKET_CLOSE_TOO_BIG", "1009"),
    ("(gint) SOUP_WEBSOCKET_CLOSE_UNSUPPORTED_DATA", "1003"),
    ("(gint) SOUP_WEBSOCKET_CONNECTION_CLIENT", "1"),
    ("(gint) SOUP_WEBSOCKET_CONNECTION_SERVER", "2"),
    ("(gint) SOUP_WEBSOCKET_CONNECTION_UNKNOWN", "0"),
    ("(gint) SOUP_WEBSOCKET_DATA_BINARY", "2"),
    ("(gint) SOUP_WEBSOCKET_DATA_TEXT", "1"),
    ("(gint) SOUP_WEBSOCKET_ERROR_BAD_HANDSHAKE", "2"),
    ("(gint) SOUP_WEBSOCKET_ERROR_BAD_ORIGIN", "3"),
    ("(gint) SOUP_WEBSOCKET_ERROR_FAILED", "0"),
    ("(gint) SOUP_WEBSOCKET_ERROR_NOT_WEBSOCKET", "1"),
    ("(gint) SOUP_WEBSOCKET_STATE_CLOSED", "3"),
    ("(gint) SOUP_WEBSOCKET_STATE_CLOSING", "2"),
    ("(gint) SOUP_WEBSOCKET_STATE_OPEN", "1"),
    ("(gint) SOUP_XMLRPC_ERROR_ARGUMENTS", "0"),
    ("(gint) SOUP_XMLRPC_ERROR_RETVAL", "1"),
    ("(gint) SOUP_XMLRPC_FAULT_APPLICATION_ERROR", "-32500"),
    ("(gint) SOUP_XMLRPC_FAULT_PARSE_ERROR_INVALID_CHARACTER_FOR_ENCODING", "-32702"),
    ("(gint) SOUP_XMLRPC_FAULT_PARSE_ERROR_NOT_WELL_FORMED", "-32700"),
    ("(gint) SOUP_XMLRPC_FAULT_PARSE_ERROR_UNSUPPORTED_ENCODING", "-32701"),
    ("(gint) SOUP_XMLRPC_FAULT_SERVER_ERROR_INTERNAL_XML_RPC_ERROR", "-32603"),
    ("(gint) SOUP_XMLRPC_FAULT_SERVER_ERROR_INVALID_METHOD_PARAMETERS", "-32602"),
    ("(gint) SOUP_XMLRPC_FAULT_SERVER_ERROR_INVALID_XML_RPC", "-32600"),
    ("(gint) SOUP_XMLRPC_FAULT_SERVER_ERROR_REQUESTED_METHOD_NOT_FOUND", "-32601"),
    ("(gint) SOUP_XMLRPC_FAULT_SYSTEM_ERROR", "-32400"),
    ("(gint) SOUP_XMLRPC_FAULT_TRANSPORT_ERROR", "-32300"),
];

