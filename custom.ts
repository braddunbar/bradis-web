type JsRespArray = {
  tag: 'array'
  value: JsRespValue[]
}

type JsRespAttribute = {
  tag: 'attribute'
  value: [JsRespValue, JsRespValue][]
}

type JsRespBignum = {
  tag: 'bignum'
  value: string
}

type JsRespBoolean = {
  tag: 'boolean'
  value: boolean
}

type JsRespDouble = {
  tag: 'double'
  value: number
}

type JsRespInt = {
  tag: 'int'
  value: number
}

type JsRespMap = {
  tag: 'map'
  value: [JsRespValue, JsRespValue][]
}

type JsRespNil = {
  tag: 'nil'
  value: null
}

type JsRespSet = {
  tag: 'set'
  value: JsRespValue[]
}

type JsRespString = {
  tag: 'string'
  value: string
}

type JsRespError = {
  tag: 'error'
  value: string
}

type JsRespPush = {
  tag: 'push'
  value: JsRespValue[]
}

type JsRespVerbatim = {
  tag: 'verbatim'
  value: {
    format: string
    value: string
  }
}

type JsRespValue =
  JsRespArray |
  JsRespAttribute |
  JsRespBignum |
  JsRespBoolean |
  JsRespDouble |
  JsRespError |
  JsRespInt |
  JsRespMap |
  JsRespNil |
  JsRespPush |
  JsRespString |
  JsRespVerbatim
