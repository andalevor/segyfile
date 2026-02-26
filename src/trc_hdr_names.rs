pub const TRC_SEQ_LINE: i32 = 0;
pub const TRC_SEQ_SEGY: i32 = 1;
pub const FFID: i32 = 2;
pub const CHAN: i32 = 3;
pub const ESP: i32 = 4;
pub const ENS_NO: i32 = 5;
pub const SEQ_NO: i32 = 6;
pub const TRACE_ID: i32 = 7;
pub const VERT_SUM: i32 = 8;
pub const HOR_SUM: i32 = 9;
pub const DATA_USE: i32 = 10;
pub const OFFSET: i32 = 11;
pub const R_ELEV: i32 = 12;
pub const S_ELEV: i32 = 13;
pub const S_DEPTH: i32 = 14;
pub const R_DATUM: i32 = 15;
pub const S_DATUM: i32 = 16;
pub const S_WATER: i32 = 17;
pub const R_WATER: i32 = 18;
pub const ELEV_SCALAR: i32 = 19;
pub const COORD_SCALAR: i32 = 20;
pub const SOU_X: i32 = 21;
pub const SOU_Y: i32 = 22;
pub const REC_X: i32 = 23;
pub const REC_Y: i32 = 24;
pub const COORD_UNITS: i32 = 25;
pub const WEATH_VEL: i32 = 26;
pub const SUBWEATH_VEL: i32 = 27;
pub const S_UPHOLE: i32 = 28;
pub const R_UPHOLE: i32 = 29;
pub const S_STAT: i32 = 30;
pub const R_STAT: i32 = 31;
pub const TOT_STAT: i32 = 32;
pub const LAG_A: i32 = 33;
pub const LAG_B: i32 = 34;
pub const DELAY_TIME: i32 = 35;
pub const MUTE_START: i32 = 36;
pub const MUTE_END: i32 = 37;
pub const SAMP_NUM: i32 = 38;
pub const SAMP_INT: i32 = 39;
pub const GAIN_TYPE: i32 = 40;
pub const GAIN_CONST: i32 = 41;
pub const INIT_GAIN: i32 = 42;
pub const CORRELATED: i32 = 43;
pub const SW_START: i32 = 44;
pub const SW_END: i32 = 45;
pub const SW_LENGTH: i32 = 46;
pub const SW_TYPE: i32 = 47;
pub const SW_TAPER_START: i32 = 48;
pub const SW_TAPER_END: i32 = 49;
pub const TAPER_TYPE: i32 = 50;
pub const ALIAS_FILT_FREQ: i32 = 51;
pub const ALIAS_FILT_SLOPE: i32 = 52;
pub const NOTCH_FILT_FREQ: i32 = 53;
pub const NOTCH_FILT_SLOPE: i32 = 54;
pub const LOW_CUT_FREQ: i32 = 55;
pub const HIGH_CUT_FREQ: i32 = 56;
pub const LOW_CUT_SLOPE: i32 = 57;
pub const HIGH_CUT_SLOPE: i32 = 58;
pub const YEAR: i32 = 59;
pub const DAY: i32 = 60;
pub const HOUR: i32 = 61;
pub const MINUTE: i32 = 62;
pub const SECOND: i32 = 63;
pub const TIME_BASIS_CODE: i32 = 64;
pub const TRACE_WEIGHT: i32 = 65;
pub const GROUP_NUM_ROLL: i32 = 66;
pub const GROUP_NUM_FIRST: i32 = 67;
pub const GROUP_NUM_LAST: i32 = 68;
pub const GAP_SIZE: i32 = 69;
pub const OVER_TRAVEL: i32 = 70;
pub const CDP_X: i32 = 71;
pub const CDP_Y: i32 = 72;
pub const INLINE: i32 = 73;
pub const XLINE: i32 = 74;
pub const SP_NUM: i32 = 75;
pub const SP_NUM_SCALAR: i32 = 76;
pub const TR_VAL_UNIT: i32 = 77;
pub const TRANS_CONST_MANT: i32 = 78;
pub const TRANS_CONST_EXP: i32 = 79;
pub const TRANS_UNITS: i32 = 80;
pub const DEVICE_ID: i32 = 81;
pub const TIME_SCALAR: i32 = 82;
pub const SOURCE_TYPE: i32 = 83;
pub const SOU_V_DIR: i32 = 84;
pub const SOU_X_DIR: i32 = 85;
pub const SOU_I_DIR: i32 = 86;
pub const SOU_MEAS_MANT: i32 = 87;
pub const SOU_MEAS_EXP: i32 = 88;
pub const SOU_MEAS_UNIT: i32 = 89;
pub const EXT_TRC_SEQ_LINE: i32 = 90;
pub const EXT_TRC_SEQ_SEGY: i32 = 91;
pub const EXT_FFID: i32 = 92;
pub const EXT_ENS_NO: i32 = 93;
pub const EXT_R_ELEV: i32 = 94;
pub const R_DEPTH: i32 = 95;
pub const EXT_S_ELEV: i32 = 96;
pub const EXT_S_DEPTH: i32 = 97;
pub const EXT_R_DATUM: i32 = 98;
pub const EXT_S_DATUM: i32 = 99;
pub const EXT_S_WATER: i32 = 100;
pub const EXT_R_WATER: i32 = 101;
pub const EXT_SOU_X: i32 = 102;
pub const EXT_SOU_Y: i32 = 103;
pub const EXT_REC_X: i32 = 104;
pub const EXT_REC_Y: i32 = 105;
pub const EXT_OFFSET: i32 = 106;
pub const EXT_SAMP_NUM: i32 = 107;
pub const NANOSECOND: i32 = 108;
pub const EXT_SAMP_INT: i32 = 109;
pub const CABLE_NUM: i32 = 110;
pub const ADD_TRC_HDR_NUM: i32 = 111;
pub const LAST_TRC_FLAG: i32 = 112;
pub const EXT_CDP_X: i32 = 113;
pub const EXT_CDP_Y: i32 = 114;

pub const TO_STRING: [&str; 115] = [
    "TRC_SEQ_LINE",
    "TRC_SEQ_SEGY",
    "FFID",
    "CHAN",
    "ESP",
    "ENS_NO",
    "SEQ_NO",
    "TRACE_ID",
    "VERT_SUM",
    "HOR_SUM",
    "DATA_USE",
    "OFFSET",
    "R_ELEV",
    "S_ELEV",
    "S_DEPTH",
    "R_DATUM",
    "S_DATUM",
    "S_WATER",
    "R_WATER",
    "ELEV_SCALAR",
    "COORD_SCALAR",
    "SOU_X",
    "SOU_Y",
    "REC_X",
    "REC_Y",
    "COORD_UNITS",
    "WEATH_VEL",
    "SUBWEATH_VEL",
    "S_UPHOLE",
    "R_UPHOLE",
    "S_STAT",
    "R_STAT",
    "TOT_STAT",
    "LAG_A",
    "LAG_B",
    "DELAY_TIME",
    "MUTE_START",
    "MUTE_END",
    "SAMP_NUM",
    "SAMP_INT",
    "GAIN_TYPE",
    "GAIN_CONST",
    "INIT_GAIN",
    "CORRELATED",
    "SW_START",
    "SW_END",
    "SW_LENGTH",
    "SW_TYPE",
    "SW_TAPER_START",
    "SW_TAPER_END",
    "TAPER_TYPE",
    "ALIAS_FILT_FREQ",
    "ALIAS_FILT_SLOPE",
    "NOTCH_FILT_FREQ",
    "NOTCH_FILT_SLOPE",
    "LOW_CUT_FREQ",
    "HIGH_CUT_FREQ",
    "LOW_CUT_SLOPE",
    "HIGH_CUT_SLOPE",
    "YEAR",
    "DAY",
    "HOUR",
    "MINUTE",
    "SECOND",
    "TIME_BASIS_CODE",
    "TRACE_WEIGHT",
    "GROUP_NUM_ROLL",
    "GROUP_NUM_FIRST",
    "GROUP_NUM_LAST",
    "GAP_SIZE",
    "OVER_TRAVEL",
    "CDP_X",
    "CDP_Y",
    "INLINE",
    "XLINE",
    "SP_NUM",
    "SP_NUM_SCALAR",
    "TR_VAL_UNIT",
    "TRANS_CONST_MANT",
    "TRANS_CONST_EXP",
    "TRANS_UNITS",
    "DEVICE_ID",
    "TIME_SCALAR",
    "SOURCE_TYPE",
    "SOU_V_DIR",
    "SOU_X_DIR",
    "SOU_I_DIR",
    "SOU_MEAS_MANT",
    "SOU_MEAS_EXP",
    "SOU_MEAS_UNIT",
    "EXT_TRC_SEQ_LINE",
    "EXT_TRC_SEQ_SEGY",
    "EXT_FFID",
    "EXT_ENS_NO",
    "EXT_R_ELEV",
    "R_DEPTH",
    "EXT_S_ELEV",
    "EXT_S_DEPTH",
    "EXT_R_DATUM",
    "EXT_S_DATUM",
    "EXT_S_WATER",
    "EXT_R_WATER",
    "EXT_SOU_X",
    "EXT_SOU_Y",
    "EXT_REC_X",
    "EXT_REC_Y",
    "EXT_OFFSET",
    "EXT_SAMP_NUM",
    "NANOSECOND",
    "EXT_SAMP_INT",
    "CABLE_NUM",
    "ADD_TRC_HDR_NUM",
    "LAST_TRC_FLAG",
    "EXT_CDP_X",
    "EXT_CDP_Y",
];
