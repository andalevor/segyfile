use crate::trc_hdr_names;
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Clone)]
pub struct BinaryHeader {
    pub job_id: i32,
    pub line_num: i32,
    pub reel_num: i32,
    pub trc_num: i16,
    pub aux_trc_num: i16,
    pub samp_int: i16,
    pub samp_int_orig: i16,
    pub samp_num: i16,
    pub samp_num_orig: i16,
    pub format_code: i16,
    pub ensemble_fold: i16,
    pub trc_sort_code: i16,
    pub vert_sum_code: i16,
    pub sw_freq_at_start: i16,
    pub sw_freq_at_end: i16,
    pub sw_length: i16,
    pub sw_type_code: i16,
    pub trc_num_of_sw_ch: i16,
    pub sw_trc_taper_length_start: i16,
    pub sw_trc_taper_length_end: i16,
    pub taper_type: i16,
    pub corr_data_trc: i16,
    pub bin_gain_recov: i16,
    pub amp_recov_meth: i16,
    pub measure_sys: i16,
    pub impulse_sig_pol: i16,
    pub vib_pol_code: i16,
    pub ext_trc_num: i32,
    pub ext_aux_trc_num: i32,
    pub ext_samp_num: i32,
    pub ext_samp_int: f64,
    pub ext_samp_int_orig: f64,
    pub ext_samp_num_orig: i32,
    pub ext_ensemble_fold: i32,
    pub endianness: i32,
    pub segy_maj_ver: u8,
    pub segy_min_ver: u8,
    pub fix_length_trc_flag: i16,
    pub ext_text_hdrs_num: i16,
    pub max_add_trc_hdrs: i32,
    pub time_basis_code: i16,
    pub num_of_trcs_in_file: u64,
    pub byte_off_of_first_trc: u64,
    pub trailer_stanza_num: i32,
}

impl BinaryHeader {
    pub fn new() -> Self {
        BinaryHeader {
            job_id: 0,
            line_num: 0,
            reel_num: 0,
            trc_num: 0,
            aux_trc_num: 0,
            samp_int: 0,
            samp_int_orig: 0,
            samp_num: 0,
            samp_num_orig: 0,
            format_code: 0,
            ensemble_fold: 0,
            trc_sort_code: 0,
            vert_sum_code: 0,
            sw_freq_at_start: 0,
            sw_freq_at_end: 0,
            sw_length: 0,
            sw_type_code: 0,
            trc_num_of_sw_ch: 0,
            sw_trc_taper_length_start: 0,
            sw_trc_taper_length_end: 0,
            taper_type: 0,
            corr_data_trc: 0,
            bin_gain_recov: 0,
            amp_recov_meth: 0,
            measure_sys: 0,
            impulse_sig_pol: 0,
            vib_pol_code: 0,
            ext_trc_num: 0,
            ext_aux_trc_num: 0,
            ext_samp_num: 0,
            ext_samp_int: 0.0,
            ext_samp_int_orig: 0.0,
            ext_samp_num_orig: 0,
            ext_ensemble_fold: 0,
            endianness: 0,
            segy_maj_ver: 0,
            segy_min_ver: 0,
            fix_length_trc_flag: 0,
            ext_text_hdrs_num: 0,
            max_add_trc_hdrs: 0,
            time_basis_code: 0,
            num_of_trcs_in_file: 0,
            byte_off_of_first_trc: 0,
            trailer_stanza_num: 0,
        }
    }
}

pub const TEXT_HEADER_SIZE: usize = 3200;
pub const BIN_HEADER_SIZE: usize = 400;
pub const TRACE_HEADER_SIZE: usize = 240;

pub const DEFAULT_TEXT_HEADER: &str = "\
C 1 CLIENT                        COMPANY                       CREW NO         \
C 2 LINE            AREA                        MAP ID                          \
C 3 REEL NO           DAY-START OF REEL     YEAR      OBSERVER                  \
C 4 INSTRUMENT: MFG            MODEL            SERIAL NO                       \
C 5 DATA TRACES/RECORD        AUXILIARY TRACES/RECORD         CDP FOLD          \
C 6 SAMPLE INTERNAL         SAMPLES/TRACE       BITS/IN      BYTES/SAMPLE       \
C 7 RECORDING FORMAT        FORMAT THIS REEL        MEASUREMENT SYSTEM          \
C 8 SAMPLE CODE: FLOATING PT     FIXED PT     FIXED PT-GAIN     CORRELATED      \
C 9 GAIN  TYPE: FIXED     BINARY     FLOATING POINT     OTHER                   \
C10 FILTERS: ALIAS     HZ  NOTCH     HZ  BAND    -     HZ  SLOPE    -    DB/OCT \
C11 SOURCE: TYPE            NUMBER/POINT        POINT INTERVAL                  \
C12     PATTERN:                           LENGTH        WIDTH                  \
C13 SWEEP: START     HZ  END     HZ  LENGTH      MS  CHANNEL NO     TYPE        \
C14 TAPER: START LENGTH       MS  END LENGTH       MS  TYPE                     \
C15 SPREAD: OFFSET        MAX DISTANCE        GROUP INTERVAL                    \
C16 GEOPHONES: PER GROUP     SPACING     FREQUENCY     MFG          MODEL       \
C17     PATTERN:                           LENGTH        WIDTH                  \
C18 TRACES SORTED BY: RECORD     CDP     OTHER                                  \
C19 AMPLITUDE RECOVEY: NONE      SPHERICAL DIV       AGC    OTHER               \
C20 MAP PROJECTION                      ZONE ID       COORDINATE UNITS          \
C21 PROCESSING:                                                                 \
C22 PROCESSING:                                                                 \
C23                                                                             \
C24                                                                             \
C25                                                                             \
C26                                                                             \
C27                                                                             \
C28                                                                             \
C29                                                                             \
C30                                                                             \
C31                                                                             \
C32                                                                             \
C33                                                                             \
C34                                                                             \
C35                                                                             \
C36                                                                             \
C37                                                                             \
C38                                                                             \
C39 SEG Y REV1                                                                  \
C40 END EBCDIC                                                                  ";

#[derive(Copy, Clone)]
pub enum TrcHdrFmt {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
}

static STD_TRC_HDR_MAPS: OnceLock<HashMap<i32, (TrcHdrFmt, i32)>> = OnceLock::new();

pub fn std_trc_hdr_map() -> &'static HashMap<i32, (TrcHdrFmt, i32)> {
    STD_TRC_HDR_MAPS.get_or_init(|| {
        HashMap::from([
            (trc_hdr_names::TRC_SEQ_LINE, (TrcHdrFmt::I32, 0)),
            (trc_hdr_names::TRC_SEQ_SEGY, (TrcHdrFmt::I32, 4)),
            (trc_hdr_names::FFID, (TrcHdrFmt::I32, 8)),
            (trc_hdr_names::CHAN, (TrcHdrFmt::I32, 12)),
            (trc_hdr_names::ESP, (TrcHdrFmt::I32, 16)),
            (trc_hdr_names::ENS_NO, (TrcHdrFmt::I32, 20)),
            (trc_hdr_names::SEQ_NO, (TrcHdrFmt::I32, 24)),
            (trc_hdr_names::TRACE_ID, (TrcHdrFmt::I16, 28)),
            (trc_hdr_names::VERT_SUM, (TrcHdrFmt::I16, 30)),
            (trc_hdr_names::HOR_SUM, (TrcHdrFmt::I16, 32)),
            (trc_hdr_names::DATA_USE, (TrcHdrFmt::I16, 34)),
            (trc_hdr_names::OFFSET, (TrcHdrFmt::I32, 36)),
            (trc_hdr_names::R_ELEV, (TrcHdrFmt::I32, 40)),
            (trc_hdr_names::S_ELEV, (TrcHdrFmt::I32, 44)),
            (trc_hdr_names::S_DEPTH, (TrcHdrFmt::I32, 48)),
            (trc_hdr_names::R_DATUM, (TrcHdrFmt::I32, 52)),
            (trc_hdr_names::S_DATUM, (TrcHdrFmt::I32, 56)),
            (trc_hdr_names::S_WATER, (TrcHdrFmt::I32, 60)),
            (trc_hdr_names::R_WATER, (TrcHdrFmt::I32, 64)),
            (trc_hdr_names::ELEV_SCALAR, (TrcHdrFmt::I16, 68)),
            (trc_hdr_names::COORD_SCALAR, (TrcHdrFmt::I16, 70)),
            (trc_hdr_names::SOU_X, (TrcHdrFmt::I32, 72)),
            (trc_hdr_names::SOU_Y, (TrcHdrFmt::I32, 76)),
            (trc_hdr_names::REC_X, (TrcHdrFmt::I32, 80)),
            (trc_hdr_names::REC_Y, (TrcHdrFmt::I32, 84)),
            (trc_hdr_names::COORD_UNITS, (TrcHdrFmt::I16, 88)),
            (trc_hdr_names::WEATH_VEL, (TrcHdrFmt::I16, 90)),
            (trc_hdr_names::SUBWEATH_VEL, (TrcHdrFmt::I16, 92)),
            (trc_hdr_names::S_UPHOLE, (TrcHdrFmt::I16, 94)),
            (trc_hdr_names::R_UPHOLE, (TrcHdrFmt::I16, 96)),
            (trc_hdr_names::S_STAT, (TrcHdrFmt::I16, 98)),
            (trc_hdr_names::R_STAT, (TrcHdrFmt::I16, 100)),
            (trc_hdr_names::TOT_STAT, (TrcHdrFmt::I16, 102)),
            (trc_hdr_names::LAG_A, (TrcHdrFmt::I16, 104)),
            (trc_hdr_names::LAG_B, (TrcHdrFmt::I16, 106)),
            (trc_hdr_names::DELAY_TIME, (TrcHdrFmt::I16, 108)),
            (trc_hdr_names::MUTE_START, (TrcHdrFmt::I16, 110)),
            (trc_hdr_names::MUTE_END, (TrcHdrFmt::I16, 112)),
            (trc_hdr_names::SAMP_NUM, (TrcHdrFmt::I16, 114)),
            (trc_hdr_names::SAMP_INT, (TrcHdrFmt::I16, 116)),
            (trc_hdr_names::GAIN_TYPE, (TrcHdrFmt::I16, 118)),
            (trc_hdr_names::GAIN_CONST, (TrcHdrFmt::I16, 120)),
            (trc_hdr_names::INIT_GAIN, (TrcHdrFmt::I16, 122)),
            (trc_hdr_names::CORRELATED, (TrcHdrFmt::I16, 124)),
            (trc_hdr_names::SW_START, (TrcHdrFmt::I16, 126)),
            (trc_hdr_names::SW_END, (TrcHdrFmt::I16, 128)),
            (trc_hdr_names::SW_LENGTH, (TrcHdrFmt::I16, 130)),
            (trc_hdr_names::SW_TYPE, (TrcHdrFmt::I16, 132)),
            (trc_hdr_names::SW_TAPER_START, (TrcHdrFmt::I16, 134)),
            (trc_hdr_names::SW_TAPER_END, (TrcHdrFmt::I16, 136)),
            (trc_hdr_names::TAPER_TYPE, (TrcHdrFmt::I16, 138)),
            (trc_hdr_names::ALIAS_FILT_FREQ, (TrcHdrFmt::I16, 140)),
            (trc_hdr_names::ALIAS_FILT_SLOPE, (TrcHdrFmt::I16, 142)),
            (trc_hdr_names::NOTCH_FILT_FREQ, (TrcHdrFmt::I16, 144)),
            (trc_hdr_names::NOTCH_FILT_SLOPE, (TrcHdrFmt::I16, 146)),
            (trc_hdr_names::LOW_CUT_FREQ, (TrcHdrFmt::I16, 148)),
            (trc_hdr_names::HIGH_CUT_FREQ, (TrcHdrFmt::I16, 150)),
            (trc_hdr_names::LOW_CUT_SLOPE, (TrcHdrFmt::I16, 152)),
            (trc_hdr_names::HIGH_CUT_SLOPE, (TrcHdrFmt::I16, 154)),
            (trc_hdr_names::YEAR, (TrcHdrFmt::I16, 156)),
            (trc_hdr_names::DAY, (TrcHdrFmt::I16, 158)),
            (trc_hdr_names::HOUR, (TrcHdrFmt::I16, 160)),
            (trc_hdr_names::MINUTE, (TrcHdrFmt::I16, 162)),
            (trc_hdr_names::SECOND, (TrcHdrFmt::I16, 164)),
            (trc_hdr_names::TIME_BASIS_CODE, (TrcHdrFmt::I16, 166)),
            (trc_hdr_names::TRACE_WEIGHT, (TrcHdrFmt::I16, 168)),
            (trc_hdr_names::GROUP_NUM_ROLL, (TrcHdrFmt::I16, 170)),
            (trc_hdr_names::GROUP_NUM_FIRST, (TrcHdrFmt::I16, 172)),
            (trc_hdr_names::GROUP_NUM_LAST, (TrcHdrFmt::I16, 174)),
            (trc_hdr_names::GAP_SIZE, (TrcHdrFmt::I16, 176)),
            (trc_hdr_names::OVER_TRAVEL, (TrcHdrFmt::I16, 178)),
            (trc_hdr_names::CDP_X, (TrcHdrFmt::I32, 180)),
            (trc_hdr_names::CDP_Y, (TrcHdrFmt::I32, 184)),
            (trc_hdr_names::INLINE, (TrcHdrFmt::I32, 188)),
            (trc_hdr_names::XLINE, (TrcHdrFmt::I32, 192)),
            (trc_hdr_names::SP_NUM, (TrcHdrFmt::I32, 196)),
            (trc_hdr_names::SP_NUM_SCALAR, (TrcHdrFmt::I16, 200)),
            (trc_hdr_names::TR_VAL_UNIT, (TrcHdrFmt::I16, 202)),
            (trc_hdr_names::TRANS_CONST_MANT, (TrcHdrFmt::I32, 204)),
            (trc_hdr_names::TRANS_CONST_EXP, (TrcHdrFmt::I16, 208)),
            (trc_hdr_names::TRANS_UNITS, (TrcHdrFmt::I16, 210)),
            (trc_hdr_names::DEVICE_ID, (TrcHdrFmt::I16, 212)),
            (trc_hdr_names::TIME_SCALAR, (TrcHdrFmt::I16, 214)),
            (trc_hdr_names::SOURCE_TYPE, (TrcHdrFmt::I16, 216)),
            (trc_hdr_names::SOU_V_DIR, (TrcHdrFmt::I16, 218)),
            (trc_hdr_names::SOU_X_DIR, (TrcHdrFmt::I16, 220)),
            (trc_hdr_names::SOU_I_DIR, (TrcHdrFmt::I16, 222)),
            (trc_hdr_names::SOU_MEAS_MANT, (TrcHdrFmt::I32, 224)),
            (trc_hdr_names::SOU_MEAS_EXP, (TrcHdrFmt::I16, 228)),
            (trc_hdr_names::SOU_MEAS_UNIT, (TrcHdrFmt::I16, 230)),
            (trc_hdr_names::EXT_TRC_SEQ_LINE, (TrcHdrFmt::I64, 240)),
            (trc_hdr_names::EXT_TRC_SEQ_SEGY, (TrcHdrFmt::I64, 248)),
            (trc_hdr_names::EXT_FFID, (TrcHdrFmt::I64, 256)),
            (trc_hdr_names::EXT_ENS_NO, (TrcHdrFmt::I64, 264)),
            (trc_hdr_names::EXT_R_ELEV, (TrcHdrFmt::F64, 272)),
            (trc_hdr_names::EXT_S_DEPTH, (TrcHdrFmt::F64, 280)),
            (trc_hdr_names::EXT_S_ELEV, (TrcHdrFmt::F64, 288)),
            (trc_hdr_names::EXT_S_DEPTH, (TrcHdrFmt::F64, 296)),
            (trc_hdr_names::EXT_R_DATUM, (TrcHdrFmt::F64, 304)),
            (trc_hdr_names::EXT_S_DATUM, (TrcHdrFmt::F64, 312)),
            (trc_hdr_names::EXT_S_WATER, (TrcHdrFmt::F64, 320)),
            (trc_hdr_names::EXT_R_WATER, (TrcHdrFmt::F64, 328)),
            (trc_hdr_names::EXT_SOU_X, (TrcHdrFmt::F64, 336)),
            (trc_hdr_names::EXT_SOU_Y, (TrcHdrFmt::F64, 344)),
            (trc_hdr_names::EXT_REC_X, (TrcHdrFmt::F64, 352)),
            (trc_hdr_names::EXT_REC_Y, (TrcHdrFmt::F64, 360)),
            (trc_hdr_names::EXT_OFFSET, (TrcHdrFmt::F64, 368)),
            (trc_hdr_names::EXT_SAMP_NUM, (TrcHdrFmt::I32, 376)),
            (trc_hdr_names::NANOSECOND, (TrcHdrFmt::I32, 380)),
            (trc_hdr_names::EXT_SAMP_INT, (TrcHdrFmt::F64, 384)),
            (trc_hdr_names::CABLE_NUM, (TrcHdrFmt::I32, 392)),
            (trc_hdr_names::ADD_TRC_HDR_NUM, (TrcHdrFmt::I16, 396)),
            (trc_hdr_names::LAST_TRC_FLAG, (TrcHdrFmt::I16, 398)),
            (trc_hdr_names::EXT_CDP_X, (TrcHdrFmt::F64, 400)),
            (trc_hdr_names::EXT_CDP_Y, (TrcHdrFmt::F64, 408)),
        ])
    })
}

pub trait Primitive: Sized {
    fn from_i8(val: i8) -> Self;
    fn from_u8(val: u8) -> Self;
    fn from_i16(val: i16) -> Self;
    fn from_u16(val: u16) -> Self;
    fn from_i32(val: i32) -> Self;
    fn from_u32(val: u32) -> Self;
    fn from_i64(val: i64) -> Self;
    fn from_u64(val: u64) -> Self;
    fn from_f32(val: f32) -> Self;
    fn from_f64(val: f64) -> Self;
    fn as_i8(val: Self) -> i8;
    fn as_u8(val: Self) -> u8;
    fn as_i16(val: Self) -> i16;
    fn as_u16(val: Self) -> u16;
    fn as_i32(val: Self) -> i32;
    fn as_u32(val: Self) -> u32;
    fn as_i64(val: Self) -> i64;
    fn as_u64(val: Self) -> u64;
    fn as_f32(val: Self) -> f32;
    fn as_f64(val: Self) -> f64;
}

macro_rules! impl_primitive {
    ($($t:ty),*) => {
        $(
            impl Primitive for $t {
                fn from_i8(val: i8) -> Self {
                    val as $t
                }
                fn from_u8(val: u8) -> Self {
                    val as $t
                }
                fn from_i16(val: i16) -> Self {
                    val as $t
                }
                fn from_u16(val: u16) -> Self {
                    val as $t
                }
                fn from_i32(val: i32) -> Self {
                    val as $t
                }
                fn from_u32(val: u32) -> Self {
                    val as $t
                }
                fn from_i64(val: i64) -> Self {
                    val as $t
                }
                fn from_u64(val: u64) -> Self {
                    val as $t
                }
                fn from_f32(val: f32) -> Self {
                    val as $t
                }
                fn from_f64(val: f64) -> Self {
                    val as $t
                }
                fn as_i8(val: $t) -> i8 {
                    val as i8
                }
                fn as_u8(val: $t) -> u8 {
                    val as u8
                }
                fn as_i16(val: $t) -> i16 {
                    val as i16
                }
                fn as_u16(val: $t) -> u16 {
                    val as u16
                }
                fn as_i32(val: $t) -> i32 {
                    val as i32
                }
                fn as_u32(val: $t) -> u32 {
                    val as u32
                }
                fn as_i64(val: $t) -> i64 {
                    val as i64
                }
                fn as_u64(val: $t) -> u64 {
                    val as u64
                }
                fn as_f32(val: $t) -> f32 {
                    val as f32
                }
                fn as_f64(val: $t) -> f64 {
                    val as f64
                }
            }
        )*
    };
}

impl_primitive!(i8, i16, i32, i64, u8, u16, u32, u64, f32, f64);
