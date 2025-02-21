use bradis::{Server, run_until_stalled};
use futures::{
    executor::block_on,
    stream::BoxStream,
    task::{Context, noop_waker_ref},
};
use respite::{RespError, RespPrimitive, RespReader, RespValue, RespWriter};
use std::task::Poll;
use tokio::io::{DuplexStream, WriteHalf, duplex, split};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Client {
    values: BoxStream<'static, Result<RespValue, RespError>>,
    writer: RespWriter<WriteHalf<DuplexStream>>,
}

#[wasm_bindgen]
impl Client {
    fn new(stream: DuplexStream) -> Self {
        let (reader, writer) = split(stream);
        let reader = RespReader::new(reader, Default::default());
        let values = reader.values();
        let writer = RespWriter::new(writer);
        Self { values, writer }
    }

    #[wasm_bindgen]
    pub fn write_inline(&mut self, cmd: String) {
        _ = block_on(self.writer.write_inline(cmd.as_bytes()));
    }

    #[wasm_bindgen]
    pub fn poll(&mut self) -> Option<JsRespValue> {
        let mut context = Context::from_waker(noop_waker_ref());
        match self.values.as_mut().poll_next(&mut context) {
            Poll::Ready(Some(value)) => Some(value.expect("error getting response").into()),
            _ => None,
        }
    }
}

#[wasm_bindgen(typescript_custom_section)]
const CUSTOM_TYPESCRIPT: &'static str = include_str!("../custom.ts");

#[wasm_bindgen(skip_typescript)]
pub struct JsRespValue {
    #[wasm_bindgen]
    tag: &'static str,

    #[wasm_bindgen]
    value: JsValue,
}

#[wasm_bindgen]
impl JsRespValue {
    #[wasm_bindgen(skip_typescript, getter = "tag")]
    pub fn tag(&self) -> String {
        self.tag.into()
    }

    #[wasm_bindgen(skip_typescript, getter = "value")]
    pub fn value(&self) -> JsValue {
        self.value.clone()
    }
}

impl From<RespPrimitive> for JsRespValue {
    fn from(value: RespPrimitive) -> Self {
        use RespPrimitive::*;
        match value {
            Integer(n) => JsRespValue {
                tag: "int",
                value: JsValue::from(n),
            },
            Nil => JsRespValue {
                tag: "nil",
                value: JsValue::NULL,
            },
            String(s) => JsRespValue {
                tag: "string",
                value: JsValue::from(format!("{}", s.escape_ascii())),
            },
        }
    }
}

#[wasm_bindgen]
pub struct VerbatimValue {
    #[wasm_bindgen]
    format: String,

    #[wasm_bindgen]
    value: String,
}

#[wasm_bindgen]
impl VerbatimValue {
    #[wasm_bindgen(getter = "format")]
    pub fn format(&self) -> String {
        self.format.clone()
    }

    #[wasm_bindgen(getter = "value")]
    pub fn value(&self) -> String {
        self.value.clone()
    }
}

impl From<RespValue> for JsRespValue {
    fn from(value: RespValue) -> Self {
        use RespValue::*;
        match value {
            Array(mut values) => JsRespValue {
                tag: "array",
                value: JsValue::from(
                    values
                        .drain(..)
                        .map(|v| v.into())
                        .collect::<Vec<JsRespValue>>(),
                ),
            },
            Attribute(map) => {
                let values: Vec<JsValue> = map
                    .iter()
                    .map(|(k, v)| {
                        JsValue::from(vec![
                            JsRespValue::from(k.clone()),
                            JsRespValue::from(v.clone()),
                        ])
                    })
                    .collect();
                JsRespValue {
                    tag: "map",
                    value: values.into(),
                }
            }
            Bignum(n) => JsRespValue {
                tag: "bignum",
                value: JsValue::from(format!("{}", std::str::from_utf8(&n).unwrap())),
            },
            Boolean(b) => JsRespValue {
                tag: "boolean",
                value: JsValue::from(b),
            },
            Double(d) => JsRespValue {
                tag: "double",
                value: JsValue::from(*d),
            },
            Error(bytes) => JsRespValue {
                tag: "error",
                value: JsValue::from(format!("{}", std::str::from_utf8(&bytes).unwrap())),
            },
            Integer(n) => JsRespValue {
                tag: "int",
                value: JsValue::from(n),
            },
            Map(map) => {
                let values: Vec<JsValue> = map
                    .iter()
                    .map(|(k, v)| {
                        JsValue::from(vec![
                            JsRespValue::from(k.clone()),
                            JsRespValue::from(v.clone()),
                        ])
                    })
                    .collect();
                JsRespValue {
                    tag: "map",
                    value: values.into(),
                }
            }
            Nil => JsRespValue {
                tag: "nil",
                value: JsValue::NULL,
            },
            Set(set) => JsRespValue {
                tag: "set",
                value: JsValue::from(
                    set.iter()
                        .map(|v| v.clone().into())
                        .collect::<Vec<JsRespValue>>(),
                ),
            },
            String(bytes) => JsRespValue {
                tag: "string",
                value: JsValue::from(format!("{}", bytes.escape_ascii())),
            },
            Push(mut values) => JsRespValue {
                tag: "push",
                value: JsValue::from(
                    values
                        .drain(..)
                        .map(|v| v.into())
                        .collect::<Vec<JsRespValue>>(),
                ),
            },
            Verbatim(format, value) => {
                let value = VerbatimValue {
                    format: format!("{}", std::str::from_utf8(&format).unwrap()),
                    value: format!("{}", std::str::from_utf8(&value).unwrap()),
                };
                JsRespValue {
                    tag: "verbatim",
                    value: value.into(),
                }
            }
        }
    }
}

#[wasm_bindgen]
pub struct Bradis {
    server: Server,
}

#[wasm_bindgen]
impl Bradis {
    #[wasm_bindgen]
    pub fn create() -> Self {
        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();

        Self {
            server: Server::default(),
        }
    }

    #[wasm_bindgen]
    pub fn tick(&mut self) {
        run_until_stalled();
    }

    #[wasm_bindgen]
    pub fn connect(&mut self) -> Client {
        let (local, remote) = duplex(100000);
        self.server.connect(remote, None);
        Client::new(local)
    }
}
