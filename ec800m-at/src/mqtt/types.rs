#[derive(Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum MqttClientIdx {
    IDX0 = 0,
    IDX1,
    IDX2,
    IDX3,
    IDX4,
    IDX5,
}

#[derive(Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum MqttVersion {
    V3_1 = 3,
    V3_1_1 = 4
}

#[derive(Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum MqttPdPCid {
    I1 = 1,
    I2,
    I3,
    I4,
    I5,
    I6,
    I7,
    I8,
    I9,
    I10,
    I11,
    I12,
    I13,
    I14,
    I15,
}

#[derive(Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum MqttSslMode {
    NormalTcp = 0,
    SslTcp = 1,
}

#[derive(Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum MqttSslCtxIdx {
    I0 = 0,
    I1,
    I2,
    I3,
    I4,
    I5
}

#[derive(Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum MqttCleanSession {
    Off = 0,
    On = 1,
}

#[derive(Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum MqttTimeOutNotice {
    Off = 0,
    On = 1,
}

#[derive(Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum MqttWillFlag {
    Off = 0,
    On = 1,
}

#[derive(Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum MqttQos {
    Qos0 = 0,
    Qos1 = 1,
    Qos2 = 2,
}

#[derive(Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum MqttWillRetain {
    Off = 0,
    On = 1,
}

#[derive(Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum MqttRecvMode {
    UrcOff = 0,
    UrcOn = 1,
}

#[derive(Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum MqttRecvLen {
    Off = 0,
    On = 1,
}

#[derive(Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum MqttSendMode {
    Str = 0,
    Hex = 1,
}

#[derive(Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum MqttViewMode {
    EchoOff = 0,
    EchoOn = 1,
}

#[derive(Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum MqttEditMode {
    TimeoutOff = 0,
    TimeoutOn = 1
}



