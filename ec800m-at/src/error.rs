pub enum Ec800mErrorCode {
    Invalid = 501,                //parameter(S) 无效参数
    OperationNotSupported = 502,  //操作不支持
    Gnss_SubsystemBusy = 503,     //GNSS 子系统繁忙
    SessionIsOngoing = 504,       //会话仍在进行
    SessionNotActive = 505,       //会话未激活
    OperationTimeout = 506,       //操作超时
    FunctionNotEnabled = 507,     //功能未使能
    TimeInformationError = 508,   //时间信息错误
    ValidityTimeOutOfRange = 512, // 已过有效期
    InternalResourceError = 513,  //内部资源错误
    GnssLocked = 514,             //GNSS 锁住
    EndByModem = 515,             //
    NotFixedNow = 516,            //当前未定位
    CMUXPortNotPpened = 517,      //CMUX 端口未打开
    UnknownError = 549,           //未知错误
}
