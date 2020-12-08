// This file was generated by gir (https://github.com/gtk-rs/gir @ 6ec2baf)
// from gir-files (https://github.com/gtk-rs/gir-files @ 2c7eeb5+)
// DO NOT EDIT

use std::env;
use std::error::Error;
use std::path::Path;
use std::mem::{align_of, size_of};
use std::process::Command;
use std::str;
use tempfile::Builder;
use aravis_sys::*;

static PACKAGES: &[&str] = &["aravis-0.8"];

#[derive(Clone, Debug)]
struct Compiler {
    pub args: Vec<String>,
}

impl Compiler {
    pub fn new() -> Result<Compiler, Box<dyn Error>> {
        let mut args = get_var("CC", "cc")?;
        args.push("-Wno-deprecated-declarations".to_owned());
        // For %z support in printf when using MinGW.
        args.push("-D__USE_MINGW_ANSI_STDIO".to_owned());
        args.extend(get_var("CFLAGS", "")?);
        args.extend(get_var("CPPFLAGS", "")?);
        args.extend(pkg_config_cflags(PACKAGES)?);
        Ok(Compiler { args })
    }

    pub fn define<'a, V: Into<Option<&'a str>>>(&mut self, var: &str, val: V) {
        let arg = match val.into() {
            None => format!("-D{}", var),
            Some(val) => format!("-D{}={}", var, val),
        };
        self.args.push(arg);
    }

    pub fn compile(&self, src: &Path, out: &Path) -> Result<(), Box<dyn Error>> {
        let mut cmd = self.to_command();
        cmd.arg(src);
        cmd.arg("-o");
        cmd.arg(out);
        let status = cmd.spawn()?.wait()?;
        if !status.success() {
            return Err(format!("compilation command {:?} failed, {}",
                               &cmd, status).into());
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
    let mut cmd = Command::new("pkg-config");
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
    /// Number of tests that failed to compile.
    failed_to_compile: usize,
}

impl Results {
    fn record_passed(&mut self) {
        self.passed += 1;
    }
    fn record_failed(&mut self) {
        self.failed += 1;
    }
    fn record_failed_to_compile(&mut self) {
        self.failed += 1;
        self.failed_to_compile += 1;
    }
    fn summary(&self) -> String {
        format!(
            "{} passed; {} failed (compilation errors: {})",
            self.passed,
            self.failed,
            self.failed_to_compile)
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
    let tmpdir = Builder::new().prefix("abi").tempdir().expect("temporary directory");
    let cc = Compiler::new().expect("configured compiler");

    assert_eq!("1",
               get_c_value(tmpdir.path(), &cc, "1").expect("C constant"),
               "failed to obtain correct constant value for 1");

    let mut results : Results = Default::default();
    for (i, &(name, rust_value)) in RUST_CONSTANTS.iter().enumerate() {
        match get_c_value(tmpdir.path(), &cc, name) {
            Err(e) => {
                results.record_failed_to_compile();
                eprintln!("{}", e);
            },
            Ok(ref c_value) => {
                if rust_value == c_value {
                    results.record_passed();
                } else {
                    results.record_failed();
                    eprintln!("Constant value mismatch for {}\nRust: {:?}\nC:    {:?}",
                              name, rust_value, c_value);
                }
            }
        };
        if (i + 1) % 25 == 0 {
            println!("constants ... {}", results.summary());
        }
    }
    results.expect_total_success();
}

#[test]
fn cross_validate_layout_with_c() {
    let tmpdir = Builder::new().prefix("abi").tempdir().expect("temporary directory");
    let cc = Compiler::new().expect("configured compiler");

    assert_eq!(Layout {size: 1, alignment: 1},
               get_c_layout(tmpdir.path(), &cc, "char").expect("C layout"),
               "failed to obtain correct layout for char type");

    let mut results : Results = Default::default();
    for (i, &(name, rust_layout)) in RUST_LAYOUTS.iter().enumerate() {
        match get_c_layout(tmpdir.path(), &cc, name) {
            Err(e) => {
                results.record_failed_to_compile();
                eprintln!("{}", e);
            },
            Ok(c_layout) => {
                if rust_layout == c_layout {
                    results.record_passed();
                } else {
                    results.record_failed();
                    eprintln!("Layout mismatch for {}\nRust: {:?}\nC:    {:?}",
                              name, rust_layout, &c_layout);
                }
            }
        };
        if (i + 1) % 25 == 0 {
            println!("layout    ... {}", results.summary());
        }
    }
    results.expect_total_success();
}

fn get_c_layout(dir: &Path, cc: &Compiler, name: &str) -> Result<Layout, Box<dyn Error>> {
    let exe = dir.join("layout");
    let mut cc = cc.clone();
    cc.define("ABI_TYPE_NAME", name);
    cc.compile(Path::new("tests/layout.c"), &exe)?;

    let mut abi_cmd = Command::new(exe);
    let output = abi_cmd.output()?;
    if !output.status.success() {
        return Err(format!("command {:?} failed, {:?}",
                           &abi_cmd, &output).into());
    }

    let stdout = str::from_utf8(&output.stdout)?;
    let mut words = stdout.trim().split_whitespace();
    let size = words.next().unwrap().parse().unwrap();
    let alignment = words.next().unwrap().parse().unwrap();
    Ok(Layout {size, alignment})
}

fn get_c_value(dir: &Path, cc: &Compiler, name: &str) -> Result<String, Box<dyn Error>> {
    let exe = dir.join("constant");
    let mut cc = cc.clone();
    cc.define("ABI_CONSTANT_NAME", name);
    cc.compile(Path::new("tests/constant.c"), &exe)?;

    let mut abi_cmd = Command::new(exe);
    let output = abi_cmd.output()?;
    if !output.status.success() {
        return Err(format!("command {:?} failed, {:?}",
                           &abi_cmd, &output).into());
    }

    let output = str::from_utf8(&output.stdout)?.trim();
    if !output.starts_with("###gir test###") ||
       !output.ends_with("###gir test###") {
        return Err(format!("command {:?} return invalid output, {:?}",
                           &abi_cmd, &output).into());
    }

    Ok(String::from(&output[14..(output.len() - 14)]))
}

const RUST_LAYOUTS: &[(&str, Layout)] = &[
    ("ArvAcquisitionMode", Layout {size: size_of::<ArvAcquisitionMode>(), alignment: align_of::<ArvAcquisitionMode>()}),
    ("ArvAuto", Layout {size: size_of::<ArvAuto>(), alignment: align_of::<ArvAuto>()}),
    ("ArvBufferClass", Layout {size: size_of::<ArvBufferClass>(), alignment: align_of::<ArvBufferClass>()}),
    ("ArvBufferPayloadType", Layout {size: size_of::<ArvBufferPayloadType>(), alignment: align_of::<ArvBufferPayloadType>()}),
    ("ArvBufferStatus", Layout {size: size_of::<ArvBufferStatus>(), alignment: align_of::<ArvBufferStatus>()}),
    ("ArvCamera", Layout {size: size_of::<ArvCamera>(), alignment: align_of::<ArvCamera>()}),
    ("ArvCameraClass", Layout {size: size_of::<ArvCameraClass>(), alignment: align_of::<ArvCameraClass>()}),
    ("ArvChunkParserClass", Layout {size: size_of::<ArvChunkParserClass>(), alignment: align_of::<ArvChunkParserClass>()}),
    ("ArvChunkParserError", Layout {size: size_of::<ArvChunkParserError>(), alignment: align_of::<ArvChunkParserError>()}),
    ("ArvDevice", Layout {size: size_of::<ArvDevice>(), alignment: align_of::<ArvDevice>()}),
    ("ArvDeviceClass", Layout {size: size_of::<ArvDeviceClass>(), alignment: align_of::<ArvDeviceClass>()}),
    ("ArvDeviceError", Layout {size: size_of::<ArvDeviceError>(), alignment: align_of::<ArvDeviceError>()}),
    ("ArvDomCharacterData", Layout {size: size_of::<ArvDomCharacterData>(), alignment: align_of::<ArvDomCharacterData>()}),
    ("ArvDomCharacterDataClass", Layout {size: size_of::<ArvDomCharacterDataClass>(), alignment: align_of::<ArvDomCharacterDataClass>()}),
    ("ArvDomDocument", Layout {size: size_of::<ArvDomDocument>(), alignment: align_of::<ArvDomDocument>()}),
    ("ArvDomDocumentClass", Layout {size: size_of::<ArvDomDocumentClass>(), alignment: align_of::<ArvDomDocumentClass>()}),
    ("ArvDomDocumentFragment", Layout {size: size_of::<ArvDomDocumentFragment>(), alignment: align_of::<ArvDomDocumentFragment>()}),
    ("ArvDomDocumentFragmentClass", Layout {size: size_of::<ArvDomDocumentFragmentClass>(), alignment: align_of::<ArvDomDocumentFragmentClass>()}),
    ("ArvDomElement", Layout {size: size_of::<ArvDomElement>(), alignment: align_of::<ArvDomElement>()}),
    ("ArvDomElementClass", Layout {size: size_of::<ArvDomElementClass>(), alignment: align_of::<ArvDomElementClass>()}),
    ("ArvDomNamedNodeMap", Layout {size: size_of::<ArvDomNamedNodeMap>(), alignment: align_of::<ArvDomNamedNodeMap>()}),
    ("ArvDomNamedNodeMapClass", Layout {size: size_of::<ArvDomNamedNodeMapClass>(), alignment: align_of::<ArvDomNamedNodeMapClass>()}),
    ("ArvDomNode", Layout {size: size_of::<ArvDomNode>(), alignment: align_of::<ArvDomNode>()}),
    ("ArvDomNodeChildListClass", Layout {size: size_of::<ArvDomNodeChildListClass>(), alignment: align_of::<ArvDomNodeChildListClass>()}),
    ("ArvDomNodeClass", Layout {size: size_of::<ArvDomNodeClass>(), alignment: align_of::<ArvDomNodeClass>()}),
    ("ArvDomNodeList", Layout {size: size_of::<ArvDomNodeList>(), alignment: align_of::<ArvDomNodeList>()}),
    ("ArvDomNodeListClass", Layout {size: size_of::<ArvDomNodeListClass>(), alignment: align_of::<ArvDomNodeListClass>()}),
    ("ArvDomNodeType", Layout {size: size_of::<ArvDomNodeType>(), alignment: align_of::<ArvDomNodeType>()}),
    ("ArvDomText", Layout {size: size_of::<ArvDomText>(), alignment: align_of::<ArvDomText>()}),
    ("ArvDomTextClass", Layout {size: size_of::<ArvDomTextClass>(), alignment: align_of::<ArvDomTextClass>()}),
    ("ArvEvaluatorClass", Layout {size: size_of::<ArvEvaluatorClass>(), alignment: align_of::<ArvEvaluatorClass>()}),
    ("ArvFakeCameraClass", Layout {size: size_of::<ArvFakeCameraClass>(), alignment: align_of::<ArvFakeCameraClass>()}),
    ("ArvFakeDeviceClass", Layout {size: size_of::<ArvFakeDeviceClass>(), alignment: align_of::<ArvFakeDeviceClass>()}),
    ("ArvFakeInterfaceClass", Layout {size: size_of::<ArvFakeInterfaceClass>(), alignment: align_of::<ArvFakeInterfaceClass>()}),
    ("ArvFakeStreamClass", Layout {size: size_of::<ArvFakeStreamClass>(), alignment: align_of::<ArvFakeStreamClass>()}),
    ("ArvGcAccessMode", Layout {size: size_of::<ArvGcAccessMode>(), alignment: align_of::<ArvGcAccessMode>()}),
    ("ArvGcBooleanClass", Layout {size: size_of::<ArvGcBooleanClass>(), alignment: align_of::<ArvGcBooleanClass>()}),
    ("ArvGcCachable", Layout {size: size_of::<ArvGcCachable>(), alignment: align_of::<ArvGcCachable>()}),
    ("ArvGcCategoryClass", Layout {size: size_of::<ArvGcCategoryClass>(), alignment: align_of::<ArvGcCategoryClass>()}),
    ("ArvGcClass", Layout {size: size_of::<ArvGcClass>(), alignment: align_of::<ArvGcClass>()}),
    ("ArvGcCommandClass", Layout {size: size_of::<ArvGcCommandClass>(), alignment: align_of::<ArvGcCommandClass>()}),
    ("ArvGcConverter", Layout {size: size_of::<ArvGcConverter>(), alignment: align_of::<ArvGcConverter>()}),
    ("ArvGcConverterClass", Layout {size: size_of::<ArvGcConverterClass>(), alignment: align_of::<ArvGcConverterClass>()}),
    ("ArvGcConverterNodeClass", Layout {size: size_of::<ArvGcConverterNodeClass>(), alignment: align_of::<ArvGcConverterNodeClass>()}),
    ("ArvGcDisplayNotation", Layout {size: size_of::<ArvGcDisplayNotation>(), alignment: align_of::<ArvGcDisplayNotation>()}),
    ("ArvGcEnumEntryClass", Layout {size: size_of::<ArvGcEnumEntryClass>(), alignment: align_of::<ArvGcEnumEntryClass>()}),
    ("ArvGcEnumerationClass", Layout {size: size_of::<ArvGcEnumerationClass>(), alignment: align_of::<ArvGcEnumerationClass>()}),
    ("ArvGcError", Layout {size: size_of::<ArvGcError>(), alignment: align_of::<ArvGcError>()}),
    ("ArvGcFeatureNode", Layout {size: size_of::<ArvGcFeatureNode>(), alignment: align_of::<ArvGcFeatureNode>()}),
    ("ArvGcFeatureNodeClass", Layout {size: size_of::<ArvGcFeatureNodeClass>(), alignment: align_of::<ArvGcFeatureNodeClass>()}),
    ("ArvGcFloatInterface", Layout {size: size_of::<ArvGcFloatInterface>(), alignment: align_of::<ArvGcFloatInterface>()}),
    ("ArvGcFloatNodeClass", Layout {size: size_of::<ArvGcFloatNodeClass>(), alignment: align_of::<ArvGcFloatNodeClass>()}),
    ("ArvGcFloatRegNode", Layout {size: size_of::<ArvGcFloatRegNode>(), alignment: align_of::<ArvGcFloatRegNode>()}),
    ("ArvGcFloatRegNodeClass", Layout {size: size_of::<ArvGcFloatRegNodeClass>(), alignment: align_of::<ArvGcFloatRegNodeClass>()}),
    ("ArvGcGroupNodeClass", Layout {size: size_of::<ArvGcGroupNodeClass>(), alignment: align_of::<ArvGcGroupNodeClass>()}),
    ("ArvGcIndexNodeClass", Layout {size: size_of::<ArvGcIndexNodeClass>(), alignment: align_of::<ArvGcIndexNodeClass>()}),
    ("ArvGcIntConverterNodeClass", Layout {size: size_of::<ArvGcIntConverterNodeClass>(), alignment: align_of::<ArvGcIntConverterNodeClass>()}),
    ("ArvGcIntRegNode", Layout {size: size_of::<ArvGcIntRegNode>(), alignment: align_of::<ArvGcIntRegNode>()}),
    ("ArvGcIntRegNodeClass", Layout {size: size_of::<ArvGcIntRegNodeClass>(), alignment: align_of::<ArvGcIntRegNodeClass>()}),
    ("ArvGcIntSwissKnifeNode", Layout {size: size_of::<ArvGcIntSwissKnifeNode>(), alignment: align_of::<ArvGcIntSwissKnifeNode>()}),
    ("ArvGcIntSwissKnifeNodeClass", Layout {size: size_of::<ArvGcIntSwissKnifeNodeClass>(), alignment: align_of::<ArvGcIntSwissKnifeNodeClass>()}),
    ("ArvGcIntegerInterface", Layout {size: size_of::<ArvGcIntegerInterface>(), alignment: align_of::<ArvGcIntegerInterface>()}),
    ("ArvGcIntegerNodeClass", Layout {size: size_of::<ArvGcIntegerNodeClass>(), alignment: align_of::<ArvGcIntegerNodeClass>()}),
    ("ArvGcInvalidatorNodeClass", Layout {size: size_of::<ArvGcInvalidatorNodeClass>(), alignment: align_of::<ArvGcInvalidatorNodeClass>()}),
    ("ArvGcIsLinear", Layout {size: size_of::<ArvGcIsLinear>(), alignment: align_of::<ArvGcIsLinear>()}),
    ("ArvGcMaskedIntRegNode", Layout {size: size_of::<ArvGcMaskedIntRegNode>(), alignment: align_of::<ArvGcMaskedIntRegNode>()}),
    ("ArvGcMaskedIntRegNodeClass", Layout {size: size_of::<ArvGcMaskedIntRegNodeClass>(), alignment: align_of::<ArvGcMaskedIntRegNodeClass>()}),
    ("ArvGcNameSpace", Layout {size: size_of::<ArvGcNameSpace>(), alignment: align_of::<ArvGcNameSpace>()}),
    ("ArvGcNode", Layout {size: size_of::<ArvGcNode>(), alignment: align_of::<ArvGcNode>()}),
    ("ArvGcNodeClass", Layout {size: size_of::<ArvGcNodeClass>(), alignment: align_of::<ArvGcNodeClass>()}),
    ("ArvGcPortClass", Layout {size: size_of::<ArvGcPortClass>(), alignment: align_of::<ArvGcPortClass>()}),
    ("ArvGcPropertyNode", Layout {size: size_of::<ArvGcPropertyNode>(), alignment: align_of::<ArvGcPropertyNode>()}),
    ("ArvGcPropertyNodeClass", Layout {size: size_of::<ArvGcPropertyNodeClass>(), alignment: align_of::<ArvGcPropertyNodeClass>()}),
    ("ArvGcPropertyNodeType", Layout {size: size_of::<ArvGcPropertyNodeType>(), alignment: align_of::<ArvGcPropertyNodeType>()}),
    ("ArvGcRegisterDescriptionNodeClass", Layout {size: size_of::<ArvGcRegisterDescriptionNodeClass>(), alignment: align_of::<ArvGcRegisterDescriptionNodeClass>()}),
    ("ArvGcRegisterInterface", Layout {size: size_of::<ArvGcRegisterInterface>(), alignment: align_of::<ArvGcRegisterInterface>()}),
    ("ArvGcRegisterNode", Layout {size: size_of::<ArvGcRegisterNode>(), alignment: align_of::<ArvGcRegisterNode>()}),
    ("ArvGcRegisterNodeClass", Layout {size: size_of::<ArvGcRegisterNodeClass>(), alignment: align_of::<ArvGcRegisterNodeClass>()}),
    ("ArvGcRepresentation", Layout {size: size_of::<ArvGcRepresentation>(), alignment: align_of::<ArvGcRepresentation>()}),
    ("ArvGcSelectorInterface", Layout {size: size_of::<ArvGcSelectorInterface>(), alignment: align_of::<ArvGcSelectorInterface>()}),
    ("ArvGcSignedness", Layout {size: size_of::<ArvGcSignedness>(), alignment: align_of::<ArvGcSignedness>()}),
    ("ArvGcStringInterface", Layout {size: size_of::<ArvGcStringInterface>(), alignment: align_of::<ArvGcStringInterface>()}),
    ("ArvGcStringRegNode", Layout {size: size_of::<ArvGcStringRegNode>(), alignment: align_of::<ArvGcStringRegNode>()}),
    ("ArvGcStringRegNodeClass", Layout {size: size_of::<ArvGcStringRegNodeClass>(), alignment: align_of::<ArvGcStringRegNodeClass>()}),
    ("ArvGcStructEntryNodeClass", Layout {size: size_of::<ArvGcStructEntryNodeClass>(), alignment: align_of::<ArvGcStructEntryNodeClass>()}),
    ("ArvGcStructRegNode", Layout {size: size_of::<ArvGcStructRegNode>(), alignment: align_of::<ArvGcStructRegNode>()}),
    ("ArvGcStructRegNodeClass", Layout {size: size_of::<ArvGcStructRegNodeClass>(), alignment: align_of::<ArvGcStructRegNodeClass>()}),
    ("ArvGcSwissKnife", Layout {size: size_of::<ArvGcSwissKnife>(), alignment: align_of::<ArvGcSwissKnife>()}),
    ("ArvGcSwissKnifeClass", Layout {size: size_of::<ArvGcSwissKnifeClass>(), alignment: align_of::<ArvGcSwissKnifeClass>()}),
    ("ArvGcSwissKnifeNode", Layout {size: size_of::<ArvGcSwissKnifeNode>(), alignment: align_of::<ArvGcSwissKnifeNode>()}),
    ("ArvGcSwissKnifeNodeClass", Layout {size: size_of::<ArvGcSwissKnifeNodeClass>(), alignment: align_of::<ArvGcSwissKnifeNodeClass>()}),
    ("ArvGcValueIndexedNodeClass", Layout {size: size_of::<ArvGcValueIndexedNodeClass>(), alignment: align_of::<ArvGcValueIndexedNodeClass>()}),
    ("ArvGcVisibility", Layout {size: size_of::<ArvGcVisibility>(), alignment: align_of::<ArvGcVisibility>()}),
    ("ArvGvDeviceClass", Layout {size: size_of::<ArvGvDeviceClass>(), alignment: align_of::<ArvGvDeviceClass>()}),
    ("ArvGvFakeCameraClass", Layout {size: size_of::<ArvGvFakeCameraClass>(), alignment: align_of::<ArvGvFakeCameraClass>()}),
    ("ArvGvInterfaceClass", Layout {size: size_of::<ArvGvInterfaceClass>(), alignment: align_of::<ArvGvInterfaceClass>()}),
    ("ArvGvPacketSizeAdjustment", Layout {size: size_of::<ArvGvPacketSizeAdjustment>(), alignment: align_of::<ArvGvPacketSizeAdjustment>()}),
    ("ArvGvStreamClass", Layout {size: size_of::<ArvGvStreamClass>(), alignment: align_of::<ArvGvStreamClass>()}),
    ("ArvGvStreamOption", Layout {size: size_of::<ArvGvStreamOption>(), alignment: align_of::<ArvGvStreamOption>()}),
    ("ArvGvStreamPacketResend", Layout {size: size_of::<ArvGvStreamPacketResend>(), alignment: align_of::<ArvGvStreamPacketResend>()}),
    ("ArvGvStreamSocketBuffer", Layout {size: size_of::<ArvGvStreamSocketBuffer>(), alignment: align_of::<ArvGvStreamSocketBuffer>()}),
    ("ArvInterface", Layout {size: size_of::<ArvInterface>(), alignment: align_of::<ArvInterface>()}),
    ("ArvInterfaceClass", Layout {size: size_of::<ArvInterfaceClass>(), alignment: align_of::<ArvInterfaceClass>()}),
    ("ArvPixelFormat", Layout {size: size_of::<ArvPixelFormat>(), alignment: align_of::<ArvPixelFormat>()}),
    ("ArvRegisterCachePolicy", Layout {size: size_of::<ArvRegisterCachePolicy>(), alignment: align_of::<ArvRegisterCachePolicy>()}),
    ("ArvStream", Layout {size: size_of::<ArvStream>(), alignment: align_of::<ArvStream>()}),
    ("ArvStreamCallbackType", Layout {size: size_of::<ArvStreamCallbackType>(), alignment: align_of::<ArvStreamCallbackType>()}),
    ("ArvStreamClass", Layout {size: size_of::<ArvStreamClass>(), alignment: align_of::<ArvStreamClass>()}),
    ("ArvUvDeviceClass", Layout {size: size_of::<ArvUvDeviceClass>(), alignment: align_of::<ArvUvDeviceClass>()}),
    ("ArvUvInterfaceClass", Layout {size: size_of::<ArvUvInterfaceClass>(), alignment: align_of::<ArvUvInterfaceClass>()}),
    ("ArvUvStreamClass", Layout {size: size_of::<ArvUvStreamClass>(), alignment: align_of::<ArvUvStreamClass>()}),
    ("ArvXmlSchemaClass", Layout {size: size_of::<ArvXmlSchemaClass>(), alignment: align_of::<ArvXmlSchemaClass>()}),
    ("ArvXmlSchemaError", Layout {size: size_of::<ArvXmlSchemaError>(), alignment: align_of::<ArvXmlSchemaError>()}),
];

const RUST_CONSTANTS: &[(&str, &str)] = &[
    ("(gint) ARV_ACQUISITION_MODE_CONTINUOUS", "0"),
    ("(gint) ARV_ACQUISITION_MODE_MULTI_FRAME", "2"),
    ("(gint) ARV_ACQUISITION_MODE_SINGLE_FRAME", "1"),
    ("(gint) ARV_AUTO_CONTINUOUS", "2"),
    ("(gint) ARV_AUTO_OFF", "0"),
    ("(gint) ARV_AUTO_ONCE", "1"),
    ("(gint) ARV_BUFFER_PAYLOAD_TYPE_CHUNK_DATA", "4"),
    ("(gint) ARV_BUFFER_PAYLOAD_TYPE_EXTENDED_CHUNK_DATA", "5"),
    ("(gint) ARV_BUFFER_PAYLOAD_TYPE_FILE", "3"),
    ("(gint) ARV_BUFFER_PAYLOAD_TYPE_H264", "8"),
    ("(gint) ARV_BUFFER_PAYLOAD_TYPE_IMAGE", "1"),
    ("(gint) ARV_BUFFER_PAYLOAD_TYPE_IMAGE_EXTENDED_CHUNK", "16385"),
    ("(gint) ARV_BUFFER_PAYLOAD_TYPE_JPEG", "6"),
    ("(gint) ARV_BUFFER_PAYLOAD_TYPE_JPEG2000", "7"),
    ("(gint) ARV_BUFFER_PAYLOAD_TYPE_MULTIZONE_IMAGE", "9"),
    ("(gint) ARV_BUFFER_PAYLOAD_TYPE_RAWDATA", "2"),
    ("(gint) ARV_BUFFER_PAYLOAD_TYPE_UNKNOWN", "-1"),
    ("(gint) ARV_BUFFER_STATUS_ABORTED", "7"),
    ("(gint) ARV_BUFFER_STATUS_CLEARED", "1"),
    ("(gint) ARV_BUFFER_STATUS_FILLING", "6"),
    ("(gint) ARV_BUFFER_STATUS_MISSING_PACKETS", "3"),
    ("(gint) ARV_BUFFER_STATUS_SIZE_MISMATCH", "5"),
    ("(gint) ARV_BUFFER_STATUS_SUCCESS", "0"),
    ("(gint) ARV_BUFFER_STATUS_TIMEOUT", "2"),
    ("(gint) ARV_BUFFER_STATUS_UNKNOWN", "-1"),
    ("(gint) ARV_BUFFER_STATUS_WRONG_PACKET_ID", "4"),
    ("(gint) ARV_CHUNK_PARSER_ERROR_BUFFER_NOT_FOUND", "1"),
    ("(gint) ARV_CHUNK_PARSER_ERROR_CHUNK_NOT_FOUND", "2"),
    ("(gint) ARV_CHUNK_PARSER_ERROR_INVALID_FEATURE_TYPE", "0"),
    ("(gint) ARV_DEVICE_ERROR_FEATURE_NOT_FOUND", "1"),
    ("(gint) ARV_DEVICE_ERROR_GENICAM_NOT_FOUND", "8"),
    ("(gint) ARV_DEVICE_ERROR_INVALID_PARAMETER", "7"),
    ("(gint) ARV_DEVICE_ERROR_NOT_CONNECTED", "2"),
    ("(gint) ARV_DEVICE_ERROR_NOT_CONTROLLER", "10"),
    ("(gint) ARV_DEVICE_ERROR_NOT_FOUND", "6"),
    ("(gint) ARV_DEVICE_ERROR_NO_STREAM_CHANNEL", "9"),
    ("(gint) ARV_DEVICE_ERROR_PROTOCOL_ERROR", "3"),
    ("(gint) ARV_DEVICE_ERROR_TIMEOUT", "5"),
    ("(gint) ARV_DEVICE_ERROR_TRANSFER_ERROR", "4"),
    ("(gint) ARV_DEVICE_ERROR_UNKNOWN", "11"),
    ("(gint) ARV_DEVICE_ERROR_WRONG_FEATURE", "0"),
    ("(gint) ARV_DOM_NODE_TYPE_ATTRIBUTE_NODE", "2"),
    ("(gint) ARV_DOM_NODE_TYPE_CDATA_SECTION_NODE", "4"),
    ("(gint) ARV_DOM_NODE_TYPE_COMMENT_NODE", "8"),
    ("(gint) ARV_DOM_NODE_TYPE_DOCUMENT_FRAGMENT_NODE", "11"),
    ("(gint) ARV_DOM_NODE_TYPE_DOCUMENT_NODE", "9"),
    ("(gint) ARV_DOM_NODE_TYPE_DOCUMENT_TYPE_NODE", "10"),
    ("(gint) ARV_DOM_NODE_TYPE_ELEMENT_NODE", "1"),
    ("(gint) ARV_DOM_NODE_TYPE_ENTITY_NODE", "6"),
    ("(gint) ARV_DOM_NODE_TYPE_ENTITY_REFERENCE_NODE", "5"),
    ("(gint) ARV_DOM_NODE_TYPE_NOTATION_NODE", "12"),
    ("(gint) ARV_DOM_NODE_TYPE_PROCESSING_INSTRUCTION_NODE", "7"),
    ("(gint) ARV_DOM_NODE_TYPE_TEXT_NODE", "3"),
    ("ARV_FAKE_CAMERA_ACQUISITION_FRAME_RATE_DEFAULT", "25.000000"),
    ("ARV_FAKE_CAMERA_BINNING_HORIZONTAL_DEFAULT", "1"),
    ("ARV_FAKE_CAMERA_BINNING_VERTICAL_DEFAULT", "1"),
    ("ARV_FAKE_CAMERA_EXPOSURE_TIME_US_DEFAULT", "10000.000000"),
    ("ARV_FAKE_CAMERA_HEIGHT_DEFAULT", "512"),
    ("ARV_FAKE_CAMERA_MEMORY_SIZE", "65536"),
    ("ARV_FAKE_CAMERA_REGISTER_ACQUISITION", "292"),
    ("ARV_FAKE_CAMERA_REGISTER_ACQUISITION_FRAME_PERIOD_US", "312"),
    ("ARV_FAKE_CAMERA_REGISTER_ACQUISITION_MODE", "300"),
    ("ARV_FAKE_CAMERA_REGISTER_ACQUISITION_START_OFFSET", "32"),
    ("ARV_FAKE_CAMERA_REGISTER_BINNING_HORIZONTAL", "264"),
    ("ARV_FAKE_CAMERA_REGISTER_BINNING_VERTICAL", "268"),
    ("ARV_FAKE_CAMERA_REGISTER_EXPOSURE_TIME_US", "288"),
    ("ARV_FAKE_CAMERA_REGISTER_FRAME_START_OFFSET", "0"),
    ("ARV_FAKE_CAMERA_REGISTER_GAIN_MODE", "276"),
    ("ARV_FAKE_CAMERA_REGISTER_GAIN_RAW", "272"),
    ("ARV_FAKE_CAMERA_REGISTER_HEIGHT", "260"),
    ("ARV_FAKE_CAMERA_REGISTER_PIXEL_FORMAT", "296"),
    ("ARV_FAKE_CAMERA_REGISTER_SENSOR_HEIGHT", "280"),
    ("ARV_FAKE_CAMERA_REGISTER_SENSOR_WIDTH", "284"),
    ("ARV_FAKE_CAMERA_REGISTER_TEST", "496"),
    ("ARV_FAKE_CAMERA_REGISTER_TRIGGER_ACTIVATION", "776"),
    ("ARV_FAKE_CAMERA_REGISTER_TRIGGER_MODE", "768"),
    ("ARV_FAKE_CAMERA_REGISTER_TRIGGER_SOURCE", "772"),
    ("ARV_FAKE_CAMERA_REGISTER_WIDTH", "256"),
    ("ARV_FAKE_CAMERA_REGISTER_X_OFFSET", "304"),
    ("ARV_FAKE_CAMERA_REGISTER_Y_OFFSET", "308"),
    ("ARV_FAKE_CAMERA_SENSOR_HEIGHT", "2048"),
    ("ARV_FAKE_CAMERA_SENSOR_WIDTH", "2048"),
    ("ARV_FAKE_CAMERA_TEST_REGISTER_DEFAULT", "305419896"),
    ("ARV_FAKE_CAMERA_WIDTH_DEFAULT", "512"),
    ("(gint) ARV_GC_ACCESS_MODE_RO", "0"),
    ("(gint) ARV_GC_ACCESS_MODE_RW", "2"),
    ("(gint) ARV_GC_ACCESS_MODE_UNDEFINED", "-1"),
    ("(gint) ARV_GC_ACCESS_MODE_WO", "1"),
    ("(gint) ARV_GC_CACHABLE_NO_CACHE", "0"),
    ("(gint) ARV_GC_CACHABLE_UNDEFINED", "-1"),
    ("(gint) ARV_GC_CACHABLE_WRITE_AROUND", "2"),
    ("(gint) ARV_GC_CACHABLE_WRITE_THROUGH", "1"),
    ("(gint) ARV_GC_DISPLAY_NOTATION_AUTOMATIC", "0"),
    ("(gint) ARV_GC_DISPLAY_NOTATION_FIXED", "1"),
    ("(gint) ARV_GC_DISPLAY_NOTATION_SCIENTIFIC", "2"),
    ("(gint) ARV_GC_DISPLAY_NOTATION_UNDEFINED", "-1"),
    ("(gint) ARV_GC_ERROR_EMPTY_ENUMERATION", "3"),
    ("(gint) ARV_GC_ERROR_ENUM_ENTRY_NOT_FOUND", "8"),
    ("(gint) ARV_GC_ERROR_GET_AS_STRING_UNDEFINED", "12"),
    ("(gint) ARV_GC_ERROR_INVALID_LENGTH", "9"),
    ("(gint) ARV_GC_ERROR_INVALID_PVALUE", "2"),
    ("(gint) ARV_GC_ERROR_NODE_NOT_FOUND", "7"),
    ("(gint) ARV_GC_ERROR_NO_DEVICE_SET", "5"),
    ("(gint) ARV_GC_ERROR_NO_EVENT_IMPLEMENTATION", "6"),
    ("(gint) ARV_GC_ERROR_OUT_OF_RANGE", "4"),
    ("(gint) ARV_GC_ERROR_PROPERTY_NOT_DEFINED", "0"),
    ("(gint) ARV_GC_ERROR_PVALUE_NOT_DEFINED", "1"),
    ("(gint) ARV_GC_ERROR_READ_ONLY", "10"),
    ("(gint) ARV_GC_ERROR_SET_FROM_STRING_UNDEFINED", "11"),
    ("(gint) ARV_GC_IS_LINEAR_NO", "0"),
    ("(gint) ARV_GC_IS_LINEAR_UNDEFINED", "-1"),
    ("(gint) ARV_GC_IS_LINEAR_YES", "1"),
    ("(gint) ARV_GC_NAME_SPACE_CUSTOM", "1"),
    ("(gint) ARV_GC_NAME_SPACE_STANDARD", "0"),
    ("(gint) ARV_GC_NAME_SPACE_UNDEFINED", "-1"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_ACCESS_MODE", "24"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_ADDRESS", "2"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_BIT", "32"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_CACHABLE", "26"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_CHUNK_ID", "34"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_COMMAND_VALUE", "33"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_CONSTANT", "23"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_DESCRIPTION", "3"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_DISPLAY_NAME", "6"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_DISPLAY_NOTATION", "13"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_DISPLAY_PRECISION", "14"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_ENDIANNESS", "28"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_EVENT_ID", "35"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_EXPRESSION", "22"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_FORMULA", "19"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_FORMULA_FROM", "21"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_FORMULA_TO", "20"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_IMPOSED_ACCESS_MODE", "25"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_INCREMENT", "10"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_IS_LINEAR", "11"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_LENGTH", "18"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_LSB", "30"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_MAXIMUM", "8"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_MINIMUM", "7"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_MSB", "31"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_OFF_VALUE", "17"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_ON_VALUE", "16"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_POLLING_TIME", "27"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_P_ADDRESS", "1003"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_P_COMMAND_VALUE", "1016"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_P_FEATURE", "1001"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_P_INCREMENT", "1010"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_P_INDEX", "1011"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_P_INVALIDATOR", "1015"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_P_IS_AVAILABLE", "1006"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_P_IS_IMPLEMENTED", "1004"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_P_IS_LOCKED", "1005"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_P_LENGTH", "1012"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_P_MAXIMUM", "1009"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_P_MINIMUM", "1008"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_P_PORT", "1013"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_P_SELECTED", "1007"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_P_UNKNONW", "1000"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_P_VALUE", "1002"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_P_VALUE_DEFAULT", "1018"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_P_VALUE_INDEXED", "1017"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_P_VARIABLE", "1014"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_REPRESENTATION", "12"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_SIGN", "29"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_SLOPE", "9"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_TOOLTIP", "5"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_UNIT", "15"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_UNKNOWN", "0"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_VALUE", "1"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_VALUE_DEFAULT", "37"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_VALUE_INDEXED", "36"),
    ("(gint) ARV_GC_PROPERTY_NODE_TYPE_VISIBILITY", "4"),
    ("(gint) ARV_GC_REPRESENTATION_BOOLEAN", "2"),
    ("(gint) ARV_GC_REPRESENTATION_HEX_NUMBER", "4"),
    ("(gint) ARV_GC_REPRESENTATION_IPV4_ADDRESS", "5"),
    ("(gint) ARV_GC_REPRESENTATION_LINEAR", "0"),
    ("(gint) ARV_GC_REPRESENTATION_LOGARITHMIC", "1"),
    ("(gint) ARV_GC_REPRESENTATION_MAC_ADDRESS", "6"),
    ("(gint) ARV_GC_REPRESENTATION_PURE_NUMBER", "3"),
    ("(gint) ARV_GC_REPRESENTATION_UNDEFINED", "-1"),
    ("(gint) ARV_GC_SIGNEDNESS_SIGNED", "0"),
    ("(gint) ARV_GC_SIGNEDNESS_UNDEFINED", "-1"),
    ("(gint) ARV_GC_SIGNEDNESS_UNSIGNED", "1"),
    ("(gint) ARV_GC_VISIBILITY_BEGINNER", "3"),
    ("(gint) ARV_GC_VISIBILITY_EXPERT", "2"),
    ("(gint) ARV_GC_VISIBILITY_GURU", "1"),
    ("(gint) ARV_GC_VISIBILITY_INVISIBLE", "0"),
    ("(gint) ARV_GC_VISIBILITY_UNDEFINED", "-1"),
    ("ARV_GV_FAKE_CAMERA_DEFAULT_INTERFACE", "lo"),
    ("ARV_GV_FAKE_CAMERA_DEFAULT_SERIAL_NUMBER", "GV01"),
    ("(gint) ARV_GV_PACKET_SIZE_ADJUSTMENT_ALWAYS", "4"),
    ("(gint) ARV_GV_PACKET_SIZE_ADJUSTMENT_NEVER", "0"),
    ("(gint) ARV_GV_PACKET_SIZE_ADJUSTMENT_ONCE", "3"),
    ("(gint) ARV_GV_PACKET_SIZE_ADJUSTMENT_ON_FAILURE", "2"),
    ("(gint) ARV_GV_PACKET_SIZE_ADJUSTMENT_ON_FAILURE_ONCE", "1"),
    ("(gint) ARV_GV_STREAM_OPTION_NONE", "0"),
    ("(gint) ARV_GV_STREAM_OPTION_PACKET_SOCKET_DISABLED", "1"),
    ("(gint) ARV_GV_STREAM_PACKET_RESEND_ALWAYS", "1"),
    ("(gint) ARV_GV_STREAM_PACKET_RESEND_NEVER", "0"),
    ("(gint) ARV_GV_STREAM_SOCKET_BUFFER_AUTO", "1"),
    ("(gint) ARV_GV_STREAM_SOCKET_BUFFER_FIXED", "0"),
    ("ARV_PIXEL_FORMAT_BAYER_BG_10", "17825807"),
    ("ARV_PIXEL_FORMAT_BAYER_BG_10P", "17432658"),
    ("ARV_PIXEL_FORMAT_BAYER_BG_10_PACKED", "17563689"),
    ("ARV_PIXEL_FORMAT_BAYER_BG_12", "17825811"),
    ("ARV_PIXEL_FORMAT_BAYER_BG_12P", "17563731"),
    ("ARV_PIXEL_FORMAT_BAYER_BG_12_PACKED", "17563693"),
    ("ARV_PIXEL_FORMAT_BAYER_BG_16", "17825841"),
    ("ARV_PIXEL_FORMAT_BAYER_BG_8", "17301515"),
    ("ARV_PIXEL_FORMAT_BAYER_GB_10", "17825806"),
    ("ARV_PIXEL_FORMAT_BAYER_GB_10P", "17432660"),
    ("ARV_PIXEL_FORMAT_BAYER_GB_10_PACKED", "17563688"),
    ("ARV_PIXEL_FORMAT_BAYER_GB_12", "17825810"),
    ("ARV_PIXEL_FORMAT_BAYER_GB_12P", "17563733"),
    ("ARV_PIXEL_FORMAT_BAYER_GB_12_PACKED", "17563692"),
    ("ARV_PIXEL_FORMAT_BAYER_GB_16", "17825840"),
    ("ARV_PIXEL_FORMAT_BAYER_GB_8", "17301514"),
    ("ARV_PIXEL_FORMAT_BAYER_GR_10", "17825804"),
    ("ARV_PIXEL_FORMAT_BAYER_GR_10P", "17432662"),
    ("ARV_PIXEL_FORMAT_BAYER_GR_10_PACKED", "17563686"),
    ("ARV_PIXEL_FORMAT_BAYER_GR_12", "17825808"),
    ("ARV_PIXEL_FORMAT_BAYER_GR_12P", "17563735"),
    ("ARV_PIXEL_FORMAT_BAYER_GR_12_PACKED", "17563690"),
    ("ARV_PIXEL_FORMAT_BAYER_GR_16", "17825838"),
    ("ARV_PIXEL_FORMAT_BAYER_GR_8", "17301512"),
    ("ARV_PIXEL_FORMAT_BAYER_RG_10", "17825805"),
    ("ARV_PIXEL_FORMAT_BAYER_RG_10P", "17432664"),
    ("ARV_PIXEL_FORMAT_BAYER_RG_10_PACKED", "17563687"),
    ("ARV_PIXEL_FORMAT_BAYER_RG_12", "17825809"),
    ("ARV_PIXEL_FORMAT_BAYER_RG_12P", "17563737"),
    ("ARV_PIXEL_FORMAT_BAYER_RG_12_PACKED", "17563691"),
    ("ARV_PIXEL_FORMAT_BAYER_RG_16", "17825839"),
    ("ARV_PIXEL_FORMAT_BAYER_RG_8", "17301513"),
    ("ARV_PIXEL_FORMAT_BGRA_8_PACKED", "35651607"),
    ("ARV_PIXEL_FORMAT_BGR_10_PACKED", "36700185"),
    ("ARV_PIXEL_FORMAT_BGR_12_PACKED", "36700187"),
    ("ARV_PIXEL_FORMAT_BGR_8_PACKED", "35127317"),
    ("ARV_PIXEL_FORMAT_CUSTOM_BAYER_BG_12_PACKED", "2165047300"),
    ("ARV_PIXEL_FORMAT_CUSTOM_BAYER_BG_16", "2165309449"),
    ("ARV_PIXEL_FORMAT_CUSTOM_BAYER_GB_12_PACKED", "2165047299"),
    ("ARV_PIXEL_FORMAT_CUSTOM_BAYER_GB_16", "2165309448"),
    ("ARV_PIXEL_FORMAT_CUSTOM_BAYER_GR_12_PACKED", "2165047297"),
    ("ARV_PIXEL_FORMAT_CUSTOM_BAYER_GR_16", "2165309446"),
    ("ARV_PIXEL_FORMAT_CUSTOM_BAYER_RG_12_PACKED", "2165047298"),
    ("ARV_PIXEL_FORMAT_CUSTOM_BAYER_RG_16", "2165309447"),
    ("ARV_PIXEL_FORMAT_CUSTOM_YUV_422_YUYV_PACKED", "2182086661"),
    ("ARV_PIXEL_FORMAT_MONO_10", "17825795"),
    ("ARV_PIXEL_FORMAT_MONO_10_PACKED", "17563652"),
    ("ARV_PIXEL_FORMAT_MONO_12", "17825797"),
    ("ARV_PIXEL_FORMAT_MONO_12_PACKED", "17563654"),
    ("ARV_PIXEL_FORMAT_MONO_14", "17825829"),
    ("ARV_PIXEL_FORMAT_MONO_16", "17825799"),
    ("ARV_PIXEL_FORMAT_MONO_8", "17301505"),
    ("ARV_PIXEL_FORMAT_MONO_8_SIGNED", "17301506"),
    ("ARV_PIXEL_FORMAT_RGBA_8_PACKED", "35651606"),
    ("ARV_PIXEL_FORMAT_RGB_10_PACKED", "36700184"),
    ("ARV_PIXEL_FORMAT_RGB_10_PLANAR", "36700194"),
    ("ARV_PIXEL_FORMAT_RGB_12_PACKED", "36700186"),
    ("ARV_PIXEL_FORMAT_RGB_12_PLANAR", "36700195"),
    ("ARV_PIXEL_FORMAT_RGB_16_PLANAR", "36700196"),
    ("ARV_PIXEL_FORMAT_RGB_8_PACKED", "35127316"),
    ("ARV_PIXEL_FORMAT_RGB_8_PLANAR", "35127329"),
    ("ARV_PIXEL_FORMAT_YUV_411_PACKED", "34340894"),
    ("ARV_PIXEL_FORMAT_YUV_422_PACKED", "34603039"),
    ("ARV_PIXEL_FORMAT_YUV_422_YUYV_PACKED", "34603058"),
    ("ARV_PIXEL_FORMAT_YUV_444_PACKED", "35127328"),
    ("(gint) ARV_REGISTER_CACHE_POLICY_DEBUG", "2"),
    ("(gint) ARV_REGISTER_CACHE_POLICY_DEFAULT", "0"),
    ("(gint) ARV_REGISTER_CACHE_POLICY_DISABLE", "0"),
    ("(gint) ARV_REGISTER_CACHE_POLICY_ENABLE", "1"),
    ("(gint) ARV_STREAM_CALLBACK_TYPE_BUFFER_DONE", "3"),
    ("(gint) ARV_STREAM_CALLBACK_TYPE_EXIT", "1"),
    ("(gint) ARV_STREAM_CALLBACK_TYPE_INIT", "0"),
    ("(gint) ARV_STREAM_CALLBACK_TYPE_START_BUFFER", "2"),
    ("(gint) ARV_XML_SCHEMA_ERROR_INVALID_STRUCTURE", "0"),
];


