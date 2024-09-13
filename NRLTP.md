# The NRLTP

NRLTP (New Relic Lightweight Telemetry Protocol) is a protocol designed to send metrics or any other time-series data, from an embedded device to a collector using any network capable of transmitting datagrams.

### Format

Data is grouped in **Hunks**. An UDP datagram can contain multiple consecutive hunks, but at least one must be present. Each hunk contains only one type of data and is composed of two parts: header and body.

All sizes and offsets are in bytes unless otherwise noted.

#### Header

| Offset | Size | Value | ID | Description |
|---|---|---|---|---|
| 0 | 3 | `0xAB`, `0xBC`, `0xCD` | `MAGIC` | Hunk start marker. |
| 3 | 1 | `1` | `VERSION` | Protocol version. |
| 4 | 1 | From 0 to 255. See section [Body](#body) for currently defined data types. | `TYPE` | Type of data contained in the body. |
| 5 | 2bits | `0` little endian, `1` big endian. | `ENDIANNESS` | Ordering of multibyte numbers. |
| 5 + 2bits | 6bits | From 0 to 63 + 8. So min value is 8 and max is 71. | `HEADER_SIZE` | Header size. |
| 6 | 2 | From 0 to 65535 (*). | `BODY_SIZE` | Body size. |
| 8 | - | - | - | If `HEADER_SIZE > 8`, here comes the fields defined in future protocol versions. |

(*): Even if `BODY_SIZE` is 16 bits wide, it can't be bigger than 65519. That's because the UDP max datagram size is 65535, the UDP header is 8 bytes and the NRLTP header is also (min) 8 bytes, so 65535 - 8 - 8 = 65519.

#### Body

Currently we support the following types of data:

| Type | Description |
|---|---
| 0 | Reserved. |
| 1 | Client ID. |
| 2 | Timestamp. |
| 3 | Integer Metrics. |
| 4 | Float Metrics. |

##### Client ID:

This is the identifier of the device (microcontroller, sensor, whatever) that is sending data. The format is simple, it's just an ASCII string without null termination. Only printable characters are allowed (greater than 31 and lesser than 127).

| Offset | Size | Value | ID | Description |
|---|---|---|---|---|
| 0 | `BODY_SIZE` | ASCII string without null termination. | `CLIENT_ID` | Client ID. |

##### Timestamp:

Records a timestamp for the following data that is going to be sent. Every metric or any other sample that is sent from this moment (in the following hunks in the current datagram), has this timestamp as origin point. If no timestamp is recorded, the server will use the receiving moment. It's a 32 bits unsigned UNIX Epoch in seconds.

| Offset | Size | Value | ID | Description |
|---|---|---|---|---|
| 0 | 4 | 32 bits unsigned integer. | `TIMESTAMP` | UNIX Timestamp. |

##### Metrics:

Contains a list of metrics. Before the list of metrics, we have the metric name:

| Offset | Size | Value | ID | Description |
|---|---|---|---|---|
| 0 | 3bits | `0` gauge, `1` count. | `METRIC_TYPE` | Metric type. |
| 0 + 3bits | 5bits | From 0 to 31 + 1. So min value is 1, and max value is 32. | `METRIC_NAME_SIZE` | Metric name size. |
| 1 | `METRIC_NAME_SIZE` | ASCII string without null termination. | `METRIC_NAME` | Metric name. |

After this, it comes the list of metric values. We have 2 different formats for metrics, with integer values and with float values. Each metric value has the following format. *Note: the following offsets are relative to the end of `METRIC_NAME`*.

Integer (type `3`):

| Offset | Size | Value | ID | Description |
|---|---|---|---|---|
| 0 | 4 | 32 bits signed integer. | `METRIC_INT32_VALUE` | Metric value. |
| 4 | 2 | From 0 to 65535. | `METRIC_TIME_OFFSET` | An offset in milliseconds since last recorded timestamp. |

Float (type `4`):

| Offset | Size | Value | ID | Description |
|---|---|---|---|---|
| 0 | 4 | 32 bits float (IEEE 754). | `METRIC_FLOAT32_VALUE` | Metric value. |
| 4 | 2 | From 0 to 65535. | `METRIC_TIME_OFFSET` | An offset in milliseconds since last recorded timestamp. |

If `METRIC_TYPE` is 1 (count), it must specify a time interval. In this case the following field follows each metric:

| Offset | Size | Value | ID | Description |
|---|---|---|---|---|
| 6 | 4 | 32 bits unsigned integer. | `METRIC_INTERVAL` | Time interval in milliseconds. |
