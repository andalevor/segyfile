use crate::common::{
    BIN_HEADER_SIZE, BinaryHeader, Primitive, TEXT_HEADER_SIZE, TRACE_HEADER_SIZE, TrcHdrFmt,
    std_trc_hdr_map,
};
use crate::error::Error;
use crate::trc_hdr_names;
use crate::utils::ebcdic_to_ascii;
use std::fs::File;
use std::io::{self, Read, Seek};

macro_rules! read {
    ( $T:ty, $A:expr ) => {{
        let (int_bytes, rest) = $A.split_at(size_of::<$T>());
        *$A = rest;
        <$T>::from_le_bytes(int_bytes.try_into().unwrap())
    }};
}

pub struct Reader<T> {
    bh: BinaryHeader,
    file: File,
    read_u64: fn(&mut &[u8]) -> u64,
    read_u32: fn(&mut &[u8]) -> u32,
    read_u24: fn(&mut &[u8]) -> u32,
    read_u16: fn(&mut &[u8]) -> u16,
    read_u8: fn(&mut &[u8]) -> u8,
    read_i64: fn(&mut &[u8]) -> i64,
    read_i32: fn(&mut &[u8]) -> i32,
    read_i24: fn(&mut &[u8]) -> i32,
    read_i16: fn(&mut &[u8]) -> i16,
    read_i8: fn(&mut &[u8]) -> i8,
    skip_headers: fn(&mut Reader<T>) -> Result<(), Error>,
    read_samples: fn(&mut Reader<T>) -> Result<Vec<T>, Error>,
    read_one_sample: fn(&Reader<T>, &mut &[u8]) -> T,
    bytes_per_sample: i32,
    samp_num: i32,
    first_trc_pos: u64,
    cur_pos: u64,
    end_of_data_pos: u64,
    samp_buf: Vec<u8>,
    hdr_buf: Vec<u8>,
}

impl<T: Primitive + Clone> Reader<T> {
    pub fn open(path: &str) -> Result<Self, Error> {
        let mut file = File::open(path)?;
        file.seek(io::SeekFrom::Start(TEXT_HEADER_SIZE as u64))?;
        let mut bin_buf = [0u8; BIN_HEADER_SIZE];
        file.read_exact(&mut bin_buf)?;
        let endianness = read!(u32, &mut &bin_buf[96..96 + 4]);
        let (read_u64, read_u32, read_u24, read_u16, read_i64, read_i32, read_i24, read_i16): (
            fn(&mut &[u8]) -> u64,
            fn(&mut &[u8]) -> u32,
            fn(&mut &[u8]) -> u32,
            fn(&mut &[u8]) -> u16,
            fn(&mut &[u8]) -> i64,
            fn(&mut &[u8]) -> i32,
            fn(&mut &[u8]) -> i32,
            fn(&mut &[u8]) -> i16,
        ) = match endianness {
            0x01020304 => (
                read_u64, read_u32, read_u24, read_u16, read_i64, read_i32, read_i24, read_i16,
            ),
            0x04030201 | 0 => (
                read_u64_sw,
                read_u32_sw,
                read_u24_sw,
                read_u16_sw,
                read_i64_sw,
                read_i32_sw,
                read_i24_sw,
                read_i16_sw,
            ),
            _ => return Err(Error::UnsupportedEndianness(endianness)),
        };
        let read_u8 = read_u8;
        let read_i8 = read_i8;
        let mut ptr = &bin_buf[..];
        let mut ptr2 = &bin_buf[300..];
        let bh = BinaryHeader {
            job_id: read_i32(&mut ptr),
            line_num: read_i32(&mut ptr),
            reel_num: read_i32(&mut ptr),
            trc_num: read_i16(&mut ptr),
            aux_trc_num: read_i16(&mut ptr),
            samp_int: read_i16(&mut ptr),
            samp_int_orig: read_i16(&mut ptr),
            samp_num: read_i16(&mut ptr),
            samp_num_orig: read_i16(&mut ptr),
            format_code: read_i16(&mut ptr),
            ensemble_fold: read_i16(&mut ptr),
            trc_sort_code: read_i16(&mut ptr),
            vert_sum_code: read_i16(&mut ptr),
            sw_freq_at_start: read_i16(&mut ptr),
            sw_freq_at_end: read_i16(&mut ptr),
            sw_length: read_i16(&mut ptr),
            sw_type_code: read_i16(&mut ptr),
            trc_num_of_sw_ch: read_i16(&mut ptr),
            sw_trc_taper_length_start: read_i16(&mut ptr),
            sw_trc_taper_length_end: read_i16(&mut ptr),
            taper_type: read_i16(&mut ptr),
            corr_data_trc: read_i16(&mut ptr),
            bin_gain_recov: read_i16(&mut ptr),
            amp_recov_meth: read_i16(&mut ptr),
            measure_sys: read_i16(&mut ptr),
            impulse_sig_pol: read_i16(&mut ptr),
            vib_pol_code: read_i16(&mut ptr),
            ext_trc_num: read_i32(&mut ptr),
            ext_aux_trc_num: read_i32(&mut ptr),
            ext_samp_num: read_i32(&mut ptr),
            ext_samp_int: f64::from_bits(read_u64(&mut ptr)),
            ext_samp_int_orig: f64::from_bits(read_u64(&mut ptr)),
            ext_samp_num_orig: read_i32(&mut ptr),
            ext_ensemble_fold: read_i32(&mut ptr),
            endianness: read_i32(&mut ptr),
            segy_maj_ver: read_u8(&mut ptr2),
            segy_min_ver: read_u8(&mut ptr2),
            fix_length_trc_flag: read_i16(&mut ptr2),
            ext_text_hdrs_num: read_i16(&mut ptr2),
            max_add_trc_hdrs: read_i32(&mut ptr2),
            time_basis_code: read_i16(&mut ptr2),
            num_of_trcs_in_file: read_u64(&mut ptr2),
            byte_off_of_first_trc: read_u64(&mut ptr2),
            trailer_stanza_num: read_i32(&mut ptr2),
        };
        let read_one_sample = match bh.format_code {
            1 => Reader::sample_from_ibm_float,
            2 => Reader::sample_from_i32,
            3 => Reader::sample_from_i16,
            5 => Reader::sample_from_ieee_float,
            6 => Reader::sample_from_ieee_double,
            7 => Reader::sample_from_i24,
            8 => Reader::sample_from_i8,
            9 => Reader::sample_from_i64,
            10 => Reader::sample_from_u32,
            11 => Reader::sample_from_u16,
            12 => Reader::sample_from_u64,
            15 => Reader::sample_from_u24,
            16 => Reader::sample_from_u8,
            _ => return Err(Error::UnsupportedFormatCode(bh.format_code)),
        };
        let bytes_per_sample = match bh.format_code {
            8 | 16 => 1,
            3 | 11 => 2,
            7 | 15 => 3,
            1 | 2 | 5 | 10 => 4,
            6 | 9 | 12 => 8,
            _ => return Err(Error::UnsupportedFormatCode(bh.format_code)),
        };
        let samp_num = if bh.ext_samp_num != 0 {
            bh.ext_samp_num
        } else {
            bh.samp_num as i32
        };
        let read_samples = if bh.fix_length_trc_flag == 1 || bh.segy_maj_ver == 0 {
            Reader::read_samples_fix
        } else {
            Reader::read_samples_var
        };
        let skip_headers = if bh.fix_length_trc_flag == 1 || bh.segy_maj_ver == 0 {
            Reader::skip_headers_fix
        } else {
            Reader::skip_headers_var
        };
        let end_of_data_pos = get_end_of_data_pos(&mut file, bh.trailer_stanza_num)?;
        let first_trc_pos = get_first_trace_pos(&mut file, bh.ext_text_hdrs_num)?;
        let samp_buf = Vec::<u8>::new();
        let hdr_buf = Vec::<u8>::new();
        Ok(Reader {
            bh,
            file,
            read_u64,
            read_u32,
            read_u24,
            read_u16,
            read_u8,
            read_i64,
            read_i32,
            read_i24,
            read_i16,
            read_i8,
            skip_headers,
            read_samples,
            read_one_sample,
            bytes_per_sample,
            samp_num,
            first_trc_pos,
            cur_pos: first_trc_pos,
            end_of_data_pos,
            samp_buf,
            hdr_buf,
        })
    }
    pub fn close(self) {}
    pub fn read_raw_text_header(&mut self) -> io::Result<[u8; TEXT_HEADER_SIZE]> {
        self.file.seek(io::SeekFrom::Start(0))?;
        let mut buf = [0u8; TEXT_HEADER_SIZE];
        self.file.read_exact(&mut buf)?;
        Ok(buf)
    }
    pub fn read_text_header(&mut self) -> io::Result<[u8; TEXT_HEADER_SIZE]> {
        let mut buf = self.read_raw_text_header()?;
        if buf[0] == 0xC3u8 {
            ebcdic_to_ascii(&mut buf);
        }
        Ok(buf)
    }
    pub fn get_binary_header(&self) -> &BinaryHeader {
        &self.bh
    }
    pub fn read_traces<U: Primitive>(
        &mut self,
        hdr_names: &[i32],
    ) -> Result<(Vec<Vec<U>>, Vec<Vec<T>>), Error> {
        let hdr_map = std_trc_hdr_map();
        for hn in hdr_names {
            let (format, offset) = hdr_map[&hn];
            let format_size = match format {
                TrcHdrFmt::I8 | TrcHdrFmt::U8 => 1,
                TrcHdrFmt::I16 | TrcHdrFmt::U16 => 2,
                TrcHdrFmt::I32 | TrcHdrFmt::U32 | TrcHdrFmt::F32 => 4,
                TrcHdrFmt::I64 | TrcHdrFmt::U64 | TrcHdrFmt::F64 => 8,
            };
            if offset + format_size > TRACE_HEADER_SIZE as i32 * (self.bh.max_add_trc_hdrs + 1) {
                return Err(Error::TraceHeaderMap(*hn));
            }
        }
        self.rewind()?;
        self.samp_buf
            .resize((self.samp_num * self.bytes_per_sample) as usize, 0u8);
        let read_hdrs_from_one = if hdr_names.len() > hdr_map.len() * 5 / 100 {
            self.hdr_buf.resize(
                (self.bh.max_add_trc_hdrs + 1) as usize * TRACE_HEADER_SIZE,
                0u8,
            );
            if self.bh.fix_length_trc_flag == 1 || self.bh.segy_maj_ver == 0 {
                Reader::<T>::read_header_by_group_fix::<U>
            } else {
                Reader::<T>::read_header_by_group_var::<U>
            }
        } else {
            if self.bh.fix_length_trc_flag == 1 || self.bh.segy_maj_ver == 0 {
                Reader::<T>::read_header_by_one_fix::<U>
            } else {
                Reader::<T>::read_header_by_one_var::<U>
            }
        };
        let mut headers = Vec::new();
        let mut samples = Vec::new();
        while self.cur_pos != self.end_of_data_pos {
            headers.push(read_hdrs_from_one(self, hdr_names)?);
            samples.push((self.read_samples)(self)?);
            self.cur_pos = self.file.stream_position()?;
        }
        Ok((headers, samples))
    }
    pub fn read_samples_once(&mut self) -> Result<Vec<T>, Error> {
        self.samp_buf
            .resize((self.samp_num * self.bytes_per_sample) as usize, 0u8);
        (self.skip_headers)(self)?;
        (self.read_samples)(self)
    }
    pub fn end_of_data(&self) -> bool {
        self.cur_pos == self.end_of_data_pos
    }
    pub fn rewind(&mut self) -> Result<(), Error> {
        self.file.seek(io::SeekFrom::Start(self.first_trc_pos))?;
        self.cur_pos = self.first_trc_pos;
        Ok(())
    }
    pub fn read_headers<U: Primitive>(&mut self, hdr_names: &[i32]) -> Result<Vec<Vec<U>>, Error> {
        self.rewind()?;
        let hdr_map = std_trc_hdr_map();
        for hn in hdr_names {
            let (format, offset) = hdr_map[&hn];
            let format_size = match format {
                TrcHdrFmt::I8 | TrcHdrFmt::U8 => 1,
                TrcHdrFmt::I16 | TrcHdrFmt::U16 => 2,
                TrcHdrFmt::I32 | TrcHdrFmt::U32 | TrcHdrFmt::F32 => 4,
                TrcHdrFmt::I64 | TrcHdrFmt::U64 | TrcHdrFmt::F64 => 8,
            };
            if offset + format_size > TRACE_HEADER_SIZE as i32 * (self.bh.max_add_trc_hdrs + 1) {
                return Err(Error::TraceHeaderMap(*hn));
            }
        }
        let read_hdrs_from_one = if hdr_names.len() > hdr_map.len() * 5 / 100 {
            self.hdr_buf.resize(
                (self.bh.max_add_trc_hdrs + 1) as usize * TRACE_HEADER_SIZE,
                0u8,
            );
            if self.bh.fix_length_trc_flag == 1 || self.bh.segy_maj_ver == 0 {
                Reader::<T>::read_header_by_group_fix::<U>
            } else {
                Reader::<T>::read_header_by_group_var::<U>
            }
        } else {
            if self.bh.fix_length_trc_flag == 1 || self.bh.segy_maj_ver == 0 {
                Reader::<T>::read_header_by_one_fix::<U>
            } else {
                Reader::<T>::read_header_by_one_var::<U>
            }
        };
        if self.bh.fix_length_trc_flag == 1 || self.bh.segy_maj_ver == 0 {
            Reader::<T>::read_all_headers_fix::<U>(self, read_hdrs_from_one, hdr_names)
        } else {
            Reader::<T>::read_all_headers_var::<U>(self, read_hdrs_from_one, hdr_names)
        }
    }
    pub fn read_samples(&mut self) -> Result<Vec<Vec<T>>, Error> {
        self.samp_buf
            .resize((self.samp_num * self.bytes_per_sample) as usize, 0u8);
        self.rewind()?;
        let mut res = Vec::new();
        while self.cur_pos != self.end_of_data_pos {
            (self.skip_headers)(self)?;
            res.push((self.read_samples)(self)?)
        }
        Ok(res)
    }
    fn read_single_hdr_value<U: Primitive>(
        reader: &mut Reader<T>,
        hdr_fmt: TrcHdrFmt,
        offset: i32,
    ) -> Result<U, Error> {
        reader
            .file
            .seek(io::SeekFrom::Start(reader.cur_pos + offset as u64))?;
        match hdr_fmt {
            TrcHdrFmt::I8 => {
                let mut buf = [0u8; 1];
                reader.file.read_exact(&mut buf)?;
                let val = (reader.read_i8)(&mut &buf[..]);
                Ok(U::from_i8(val))
            }
            TrcHdrFmt::I16 => {
                let mut buf = [0u8; 2];
                reader.file.read_exact(&mut buf)?;
                let val = (reader.read_i16)(&mut &buf[..]);
                Ok(U::from_i16(val))
            }
            TrcHdrFmt::I32 => {
                let mut buf = [0u8; 4];
                reader.file.read_exact(&mut buf)?;
                let val = (reader.read_i32)(&mut &buf[..]);
                Ok(U::from_i32(val))
            }
            TrcHdrFmt::I64 => {
                let mut buf = [0u8; 8];
                reader.file.read_exact(&mut buf)?;
                let val = (reader.read_i64)(&mut &buf[..]);
                Ok(U::from_i64(val))
            }
            TrcHdrFmt::U8 => {
                let mut buf = [0u8; 1];
                reader.file.read_exact(&mut buf)?;
                let val = (reader.read_u8)(&mut &buf[..]);
                Ok(U::from_u8(val))
            }
            TrcHdrFmt::U16 => {
                let mut buf = [0u8; 2];
                reader.file.read_exact(&mut buf)?;
                let val = (reader.read_u16)(&mut &buf[..]);
                Ok(U::from_u16(val))
            }
            TrcHdrFmt::U32 => {
                let mut buf = [0u8; 4];
                reader.file.read_exact(&mut buf)?;
                let val = (reader.read_u32)(&mut &buf[..]);
                Ok(U::from_u32(val))
            }
            TrcHdrFmt::U64 => {
                let mut buf = [0u8; 8];
                reader.file.read_exact(&mut buf)?;
                let val = (reader.read_u64)(&mut &buf[..]);
                Ok(U::from_u64(val))
            }
            TrcHdrFmt::F32 => {
                let mut buf = [0u8; 4];
                reader.file.read_exact(&mut buf)?;
                let val = (reader.read_u32)(&mut &buf[..]);
                Ok(U::from_f32(f32::from_bits(val)))
            }
            TrcHdrFmt::F64 => {
                let mut buf = [0u8; 8];
                reader.file.read_exact(&mut buf)?;
                let val = (reader.read_u64)(&mut &buf[..]);
                Ok(U::from_f64(f64::from_bits(val)))
            }
        }
    }
    fn read_header_by_one_fix<U: Primitive>(
        reader: &mut Reader<T>,
        hdr_names: &[i32],
    ) -> Result<Vec<U>, Error> {
        let hdr_map = std_trc_hdr_map();
        let mut res_vec = Vec::with_capacity(hdr_names.len());
        for i in 0..hdr_names.len() {
            let (hdr_fmt, offset) = hdr_map[&hdr_names[i]];
            let res = Reader::<T>::read_single_hdr_value(reader, hdr_fmt, offset)?;
            res_vec.push(res);
        }
        reader.cur_pos = reader.file.seek(io::SeekFrom::Start(
            reader.cur_pos + (reader.bh.max_add_trc_hdrs + 1) as u64 * TRACE_HEADER_SIZE as u64,
        ))?;
        Ok(res_vec)
    }
    fn read_header_by_one_var<U: Primitive>(
        reader: &mut Reader<T>,
        hdr_names: &[i32],
    ) -> Result<Vec<U>, Error> {
        let hdr_map = std_trc_hdr_map();
        let mut res_vec = Vec::with_capacity(hdr_names.len());
        let add_trc_hdrs_num: i16 = if reader.bh.max_add_trc_hdrs > 0 {
            let (hdr_fmt, offset) = hdr_map[&trc_hdr_names::ADD_TRC_HDR_NUM];
            Reader::<T>::read_single_hdr_value(reader, hdr_fmt, offset)?
        } else {
            0
        };
        for i in 0..hdr_names.len() {
            let (hdr_fmt, offset) = hdr_map[&hdr_names[i]];
            let format_size = match hdr_fmt {
                TrcHdrFmt::I8 | TrcHdrFmt::U8 => 1,
                TrcHdrFmt::I16 | TrcHdrFmt::U16 => 2,
                TrcHdrFmt::I32 | TrcHdrFmt::U32 | TrcHdrFmt::F32 => 4,
                TrcHdrFmt::I64 | TrcHdrFmt::U64 | TrcHdrFmt::F64 => 8,
            };
            let res = if offset + format_size
                > (add_trc_hdrs_num as i32 + 1) * TRACE_HEADER_SIZE as i32
            {
                U::from_i32(0)
            } else {
                Reader::<T>::read_single_hdr_value(reader, hdr_fmt, offset)?
            };
            res_vec.push(res);
        }
        reader.cur_pos = reader.file.seek(io::SeekFrom::Start(
            reader.cur_pos + (add_trc_hdrs_num + 1) as u64 * TRACE_HEADER_SIZE as u64,
        ))?;
        Ok(res_vec)
    }
    fn read_header_by_group_fix<U: Primitive>(
        reader: &mut Reader<T>,
        hdr_names: &[i32],
    ) -> Result<Vec<U>, Error> {
        let hdr_map = std_trc_hdr_map();
        let mut res_vec = Vec::with_capacity(hdr_names.len());
        reader.file.read_exact(&mut reader.hdr_buf)?;
        for hdr_name in hdr_names {
            let (hdr_fmt, offset) = hdr_map[hdr_name];
            let res = match hdr_fmt {
                TrcHdrFmt::I8 => {
                    let val = (reader.read_i8)(&mut &reader.hdr_buf[offset as usize..]);
                    U::from_i8(val)
                }
                TrcHdrFmt::I16 => {
                    let val = (reader.read_i16)(&mut &reader.hdr_buf[offset as usize..]);
                    U::from_i16(val)
                }
                TrcHdrFmt::I32 => {
                    let val = (reader.read_i32)(&mut &reader.hdr_buf[offset as usize..]);
                    U::from_i32(val)
                }
                TrcHdrFmt::I64 => {
                    let val = (reader.read_i64)(&mut &reader.hdr_buf[offset as usize..]);
                    U::from_i64(val)
                }
                TrcHdrFmt::U8 => {
                    let val = (reader.read_u8)(&mut &reader.hdr_buf[offset as usize..]);
                    U::from_u8(val)
                }
                TrcHdrFmt::U16 => {
                    let val = (reader.read_u16)(&mut &reader.hdr_buf[offset as usize..]);
                    U::from_u16(val)
                }
                TrcHdrFmt::U32 => {
                    let val = (reader.read_u32)(&mut &reader.hdr_buf[offset as usize..]);
                    U::from_u32(val)
                }
                TrcHdrFmt::U64 => {
                    let val = (reader.read_u64)(&mut &reader.hdr_buf[offset as usize..]);
                    U::from_u64(val)
                }
                TrcHdrFmt::F32 => {
                    let val = (reader.read_u32)(&mut &reader.hdr_buf[offset as usize..]);
                    U::from_f32(f32::from_bits(val))
                }
                TrcHdrFmt::F64 => {
                    let val = (reader.read_u64)(&mut &reader.hdr_buf[offset as usize..]);
                    U::from_f64(f64::from_bits(val))
                }
            };
            res_vec.push(res);
        }
        Ok(res_vec)
    }
    fn read_header_by_group_var<U: Primitive>(
        reader: &mut Reader<T>,
        hdr_names: &[i32],
    ) -> Result<Vec<U>, Error> {
        let hdr_map = std_trc_hdr_map();
        let mut res_vec = Vec::with_capacity(hdr_names.len());
        let add_trc_hdrs_num: i16 = if reader.bh.max_add_trc_hdrs > 0 {
            let (hdr_fmt, offset) = hdr_map[&trc_hdr_names::ADD_TRC_HDR_NUM];
            Reader::<T>::read_single_hdr_value(reader, hdr_fmt, offset)?
        } else {
            0
        };
        reader
            .hdr_buf
            .resize((add_trc_hdrs_num + 1) as usize * TRACE_HEADER_SIZE, 0u8);
        reader.file.read_exact(&mut reader.hdr_buf)?;
        for hdr_name in hdr_names {
            let (hdr_fmt, offset) = hdr_map[hdr_name];
            let format_size = match hdr_fmt {
                TrcHdrFmt::I8 | TrcHdrFmt::U8 => 1,
                TrcHdrFmt::I16 | TrcHdrFmt::U16 => 2,
                TrcHdrFmt::I32 | TrcHdrFmt::U32 | TrcHdrFmt::F32 => 4,
                TrcHdrFmt::I64 | TrcHdrFmt::U64 | TrcHdrFmt::F64 => 8,
            };
            let res = if offset + format_size
                > (add_trc_hdrs_num as i32 + 1) * TRACE_HEADER_SIZE as i32
            {
                U::from_i32(0)
            } else {
                match hdr_fmt {
                    TrcHdrFmt::I8 => {
                        let val = (reader.read_i8)(&mut &reader.hdr_buf[offset as usize..]);
                        U::from_i8(val)
                    }
                    TrcHdrFmt::I16 => {
                        let val = (reader.read_i16)(&mut &reader.hdr_buf[offset as usize..]);
                        U::from_i16(val)
                    }
                    TrcHdrFmt::I32 => {
                        let val = (reader.read_i32)(&mut &reader.hdr_buf[offset as usize..]);
                        U::from_i32(val)
                    }
                    TrcHdrFmt::I64 => {
                        let val = (reader.read_i64)(&mut &reader.hdr_buf[offset as usize..]);
                        U::from_i64(val)
                    }
                    TrcHdrFmt::U8 => {
                        let val = (reader.read_u8)(&mut &reader.hdr_buf[offset as usize..]);
                        U::from_u8(val)
                    }
                    TrcHdrFmt::U16 => {
                        let val = (reader.read_u16)(&mut &reader.hdr_buf[offset as usize..]);
                        U::from_u16(val)
                    }
                    TrcHdrFmt::U32 => {
                        let val = (reader.read_u32)(&mut &reader.hdr_buf[offset as usize..]);
                        U::from_u32(val)
                    }
                    TrcHdrFmt::U64 => {
                        let val = (reader.read_u64)(&mut &reader.hdr_buf[offset as usize..]);
                        U::from_u64(val)
                    }
                    TrcHdrFmt::F32 => {
                        let val = (reader.read_u32)(&mut &reader.hdr_buf[offset as usize..]);
                        U::from_f32(f32::from_bits(val))
                    }
                    TrcHdrFmt::F64 => {
                        let val = (reader.read_u64)(&mut &reader.hdr_buf[offset as usize..]);
                        U::from_f64(f64::from_bits(val))
                    }
                }
            };
            res_vec.push(res);
        }
        Ok(res_vec)
    }
    fn read_all_headers_fix<U: Primitive>(
        reader: &mut Reader<T>,
        read_fn: fn(&mut Reader<T>, &[i32]) -> Result<Vec<U>, Error>,
        hdr_names: &[i32],
    ) -> Result<Vec<Vec<U>>, Error> {
        let mut res = Vec::new();
        while reader.cur_pos != reader.end_of_data_pos {
            res.push(read_fn(reader, hdr_names)?);
            reader.cur_pos = reader.file.seek(io::SeekFrom::Current(
                (reader.samp_num * reader.bytes_per_sample) as i64,
            ))?;
        }
        Ok(res)
    }
    fn read_all_headers_var<U: Primitive>(
        reader: &mut Reader<T>,
        read_fn: fn(&mut Reader<T>, &[i32]) -> Result<Vec<U>, Error>,
        hdr_names: &[i32],
    ) -> Result<Vec<Vec<U>>, Error> {
        let mut res = Vec::new();
        for _ in hdr_names {
            res.push(Vec::new());
        }
        while reader.cur_pos != reader.end_of_data_pos {
            let hdr_map = std_trc_hdr_map();
            let (hdr_fmt, offset) = hdr_map[&trc_hdr_names::SAMP_NUM];
            let mut samp_num: i32 = Reader::<T>::read_single_hdr_value(reader, hdr_fmt, offset)?;
            if reader.bh.max_add_trc_hdrs > 0 {
                let (hdr_fmt, offset) = hdr_map[&trc_hdr_names::EXT_SAMP_NUM];
                let ext_samp_num = Reader::<T>::read_single_hdr_value(reader, hdr_fmt, offset)?;
                if ext_samp_num > samp_num {
                    samp_num = ext_samp_num;
                }
            }
            res.push(read_fn(reader, hdr_names)?);
            reader.cur_pos = reader.file.seek(io::SeekFrom::Current(
                (samp_num * reader.bytes_per_sample) as i64,
            ))?;
        }
        Ok(res)
    }
    fn skip_headers_fix(reader: &mut Reader<T>) -> Result<(), Error> {
        reader.cur_pos = reader.file.seek(io::SeekFrom::Current(
            (reader.bh.max_add_trc_hdrs + 1) as i64 * TRACE_HEADER_SIZE as i64,
        ))?;
        Ok(())
    }
    fn read_samples_fix(reader: &mut Reader<T>) -> Result<Vec<T>, Error> {
        let mut result = vec![T::from_i32(0); reader.samp_num as usize];
        reader.file.read_exact(&mut reader.samp_buf)?;
        let mut ptr: &[u8] = &reader.samp_buf;
        for samp in result.iter_mut() {
            *samp = (reader.read_one_sample)(&reader, &mut ptr);
        }
        reader.cur_pos = reader.file.stream_position()?;
        Ok(result)
    }
    fn skip_headers_var(reader: &mut Reader<T>) -> Result<(), Error> {
        let hdr_map = std_trc_hdr_map();
        let add_trc_hdr_num = if reader.bh.max_add_trc_hdrs > 0 {
            let (hdr_fmt, offset) = hdr_map[&trc_hdr_names::ADD_TRC_HDR_NUM];
            Reader::<T>::read_single_hdr_value(reader, hdr_fmt, offset)?
        } else {
            0
        };
        reader.cur_pos = reader.file.seek(io::SeekFrom::Current(
            (add_trc_hdr_num + 1) as i64 * TRACE_HEADER_SIZE as i64,
        ))?;
        Ok(())
    }
    fn read_samples_var(reader: &mut Reader<T>) -> Result<Vec<T>, Error> {
        let hdr_map = std_trc_hdr_map();
        let (hdr_fmt, offset) = hdr_map[&trc_hdr_names::SAMP_NUM];
        let mut samp_num: i32 = Reader::<T>::read_single_hdr_value(reader, hdr_fmt, offset)?;
        if reader.bh.max_add_trc_hdrs > 0 {
            let (hdr_fmt, offset) = hdr_map[&trc_hdr_names::EXT_SAMP_NUM];
            let ext_samp_num: i32 = Reader::<T>::read_single_hdr_value(reader, hdr_fmt, offset)?;
            if ext_samp_num > samp_num {
                samp_num = ext_samp_num;
            }
        }
        if samp_num != reader.samp_num {
            reader
                .samp_buf
                .resize((samp_num * reader.bytes_per_sample) as usize, 0u8);
            reader.samp_num = samp_num;
        }
        reader.file.read_exact(&mut reader.samp_buf)?;
        let mut ptr: &[u8] = &reader.samp_buf;
        let mut result = vec![T::from_i32(0); samp_num as usize];
        for samp in result.iter_mut() {
            *samp = (reader.read_one_sample)(&reader, &mut ptr);
        }
        reader.cur_pos = reader.file.stream_position()?;
        Ok(result)
    }
    fn sample_from_ibm_float(reader: &Reader<T>, buf: &mut &[u8]) -> T {
        let bytes = (reader.read_u32)(buf);
        let sign = if bytes >> 31 == 1 { -1.0 } else { 1.0 };
        let exp = (bytes >> 24 & 0x7f) as i32;
        let fraction = bytes & 0x00ffffff;
        T::from_f64(fraction as f64 / 2f64.powi(24) * 16f64.powi(exp - 64) * sign)
    }
    fn sample_from_i32(reader: &Reader<T>, buf: &mut &[u8]) -> T {
        let value = (reader.read_i32)(buf);
        T::from_i32(value)
    }
    fn sample_from_i16(reader: &Reader<T>, buf: &mut &[u8]) -> T {
        let value = (reader.read_i16)(buf);
        T::from_i16(value)
    }
    fn sample_from_ieee_float(reader: &Reader<T>, buf: &mut &[u8]) -> T {
        let bytes = (reader.read_u32)(buf);
        T::from_f32(f32::from_bits(bytes))
    }
    fn sample_from_ieee_double(reader: &Reader<T>, buf: &mut &[u8]) -> T {
        let bytes = (reader.read_u64)(buf);
        T::from_f64(f64::from_bits(bytes))
    }
    fn sample_from_i24(reader: &Reader<T>, buf: &mut &[u8]) -> T {
        let value = (reader.read_i24)(buf);
        T::from_i32(value)
    }
    fn sample_from_i8(reader: &Reader<T>, buf: &mut &[u8]) -> T {
        let value = (reader.read_i8)(buf);
        T::from_i8(value)
    }
    fn sample_from_i64(reader: &Reader<T>, buf: &mut &[u8]) -> T {
        let value = (reader.read_i64)(buf);
        T::from_i64(value)
    }
    fn sample_from_u32(reader: &Reader<T>, buf: &mut &[u8]) -> T {
        let value = (reader.read_u32)(buf);
        T::from_u32(value)
    }
    fn sample_from_u16(reader: &Reader<T>, buf: &mut &[u8]) -> T {
        let value = (reader.read_u16)(buf);
        T::from_u16(value)
    }
    fn sample_from_u64(reader: &Reader<T>, buf: &mut &[u8]) -> T {
        let value = (reader.read_u64)(buf);
        T::from_u64(value)
    }
    fn sample_from_u24(reader: &Reader<T>, buf: &mut &[u8]) -> T {
        let value = (reader.read_u24)(buf);
        T::from_u32(value)
    }
    fn sample_from_u8(reader: &Reader<T>, buf: &mut &[u8]) -> T {
        let value = (reader.read_u8)(buf);
        T::from_u8(value)
    }
}
fn get_end_of_data_pos(file: &mut File, trailer_stanzas_num: i32) -> Result<u64, Error> {
    let mut pos = file.seek(io::SeekFrom::End(0))?;
    if trailer_stanzas_num == -1 {
        // TODO: add support of variable number of trailer stanzas
        return Err(Error::UnsupportedNumberOfStanzas(trailer_stanzas_num));
    } else {
        pos -= trailer_stanzas_num as u64 * TEXT_HEADER_SIZE as u64;
    }
    Ok(pos)
}
fn get_first_trace_pos(file: &mut File, ext_txt_hdrs_num: i16) -> Result<u64, Error> {
    let mut pos = file.seek(io::SeekFrom::Start(
        (TEXT_HEADER_SIZE + BIN_HEADER_SIZE) as u64,
    ))?;
    if ext_txt_hdrs_num == -1 {
        let mut buf = [0u8; TEXT_HEADER_SIZE];
        loop {
            file.read_exact(&mut buf)?;
            let hdr = str::from_utf8(&buf)?;
            if hdr.starts_with("((SEG: EndText))") {
                break;
            }
        }
    } else {
        pos += ext_txt_hdrs_num as u64 * TEXT_HEADER_SIZE as u64;
    }
    Ok(pos)
}
fn read_i8(buf: &mut &[u8]) -> i8 {
    read!(i8, buf)
}
fn read_u8(buf: &mut &[u8]) -> u8 {
    read!(u8, buf)
}
fn read_i16(buf: &mut &[u8]) -> i16 {
    read!(i16, buf)
}
fn read_u16(buf: &mut &[u8]) -> u16 {
    read!(u16, buf)
}
fn read_i24(buf: &mut &[u8]) -> i32 {
    ((read!(u16, buf) as u32) | (read!(u8, buf) as u32) << 16) as i32
}
fn read_u24(buf: &mut &[u8]) -> u32 {
    (read!(u16, buf) as u32) | (read!(u8, buf) as u32) << 16
}
fn read_i32(buf: &mut &[u8]) -> i32 {
    read!(i32, buf)
}
fn read_u32(buf: &mut &[u8]) -> u32 {
    read!(u32, buf)
}
fn read_i64(buf: &mut &[u8]) -> i64 {
    read!(i64, buf)
}
fn read_u64(buf: &mut &[u8]) -> u64 {
    read!(u64, buf)
}
fn read_i16_sw(buf: &mut &[u8]) -> i16 {
    read!(i16, buf).swap_bytes()
}
fn read_u16_sw(buf: &mut &[u8]) -> u16 {
    read!(u16, buf).swap_bytes()
}
fn read_i24_sw(buf: &mut &[u8]) -> i32 {
    let tmp = (read!(u16, buf) as u32) | (read!(u8, buf) as u32) << 16;
    (tmp.swap_bytes() >> 8) as i32
}
fn read_u24_sw(buf: &mut &[u8]) -> u32 {
    let tmp = (read!(u16, buf) as u32) | (read!(u8, buf) as u32) << 16;
    tmp.swap_bytes() >> 8
}
fn read_i32_sw(buf: &mut &[u8]) -> i32 {
    read!(i32, buf).swap_bytes()
}
fn read_u32_sw(buf: &mut &[u8]) -> u32 {
    read!(u32, buf).swap_bytes()
}
fn read_i64_sw(buf: &mut &[u8]) -> i64 {
    read!(i64, buf).swap_bytes()
}
fn read_u64_sw(buf: &mut &[u8]) -> u64 {
    read!(u64, buf).swap_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_header_reading() {
        let mut sgy =
            Reader::<f32>::open("samples/ieee_single.sgy").expect("Problem opening the file");
        let text_header = sgy.read_text_header().expect("Problem opening the file");
        for slice in text_header.chunks(80) {
            let s = str::from_utf8(slice).expect("Invalid UTF-8 sequence");
            println!("{}", s);
        }
        let s = std::str::from_utf8(&text_header).expect("Invalid UTF-8 sequence");
        assert_eq!(s, crate::common::DEFAULT_TEXT_HEADER);
    }
    #[test]
    fn test_bin_header_reading() {
        let sgy = Reader::<f32>::open("samples/ieee_single.sgy").expect("Problem opening the file");
        let bin_hdr = sgy.get_binary_header();
        println!("Job identification number: {}", bin_hdr.job_id);
        println!("Line number: {}", bin_hdr.line_num);
        println!("Reel number: {}", bin_hdr.reel_num);
        println!("Number of data traces per ensemble: {}", bin_hdr.trc_num);
        println!(
            "Number of auxiliary traces per ensemble: {}",
            bin_hdr.aux_trc_num
        );
        println!("Sample interval: {}", bin_hdr.samp_int);
        println!(
            "Sample interval of original field record: {}",
            bin_hdr.samp_int_orig
        );
        println!("Number of samples per data trace: {}", bin_hdr.samp_num);
        println!(
            "Number of samples per data trace of original field recording: {}",
            bin_hdr.samp_num_orig
        );
        println!("Data sample format code: {}", bin_hdr.format_code);
        println!("Ensemble fold: {}", bin_hdr.ensemble_fold);
        println!("Trace sorting code: {}", bin_hdr.trc_sort_code);
        println!("Vertical sum code: {}", bin_hdr.vert_sum_code);
        println!("Sweep frequency at start: {}", bin_hdr.sw_freq_at_start);
        println!("Sweep frequency at end: {}", bin_hdr.sw_freq_at_end);
        println!("Sweep length: {}", bin_hdr.sw_length);
        println!("Sweep type code: {}", bin_hdr.sw_type_code);
        println!(
            "Trace number of sweep channel: {}",
            bin_hdr.trc_num_of_sw_ch
        );
        println!(
            "Sweep trace taper length at start: {}",
            bin_hdr.sw_trc_taper_length_start
        );
        println!(
            "Sweep trace taper length at end: {}",
            bin_hdr.sw_trc_taper_length_end
        );
        println!("Taper type: {}", bin_hdr.taper_type);
        println!("Correlated data traces: {}", bin_hdr.corr_data_trc);
        println!("Binary gain recovered: {}", bin_hdr.bin_gain_recov);
        println!("Amplitude recovery method: {}", bin_hdr.amp_recov_meth);
        println!("Measurement system: {}", bin_hdr.measure_sys);
        println!("Impulse signal polarity: {}", bin_hdr.impulse_sig_pol);
        println!("Viratory polarity code: {}", bin_hdr.vib_pol_code);
        println!(
            "Extended number of data traces per ensemble: {}",
            bin_hdr.ext_trc_num
        );
        println!(
            "Extended number of auxiliary traces per ensemble: {}",
            bin_hdr.ext_aux_trc_num
        );
        println!(
            "Extended number of samples per data trace: {}",
            bin_hdr.ext_samp_num
        );
        println!("Extended sample interval: {}", bin_hdr.ext_samp_int);
        println!(
            "Extended sample interval of original field recording: {}",
            bin_hdr.ext_samp_int_orig
        );
        println!(
            "Extended number of samples per data trace in original recording: {}",
            bin_hdr.ext_samp_num_orig
        );
        println!("Extended ensemble fold: {}", bin_hdr.ext_ensemble_fold);
        println!("Endianness constant: {:x}", bin_hdr.ext_ensemble_fold);
        println!("SEG-Y major version: {}", bin_hdr.segy_maj_ver);
        println!("SEG-Y minor version: {}", bin_hdr.segy_min_ver);
        println!("Fixed lenght trace flag: {}", bin_hdr.fix_length_trc_flag);
        println!(
            "Number of extended textual headers: {}",
            bin_hdr.ext_text_hdrs_num
        );
        println!(
            "Maximum number of additional trace headers: {}",
            bin_hdr.max_add_trc_hdrs
        );
        println!("Time basis code: {}", bin_hdr.time_basis_code);
        println!(
            "Number of traces in this file: {}",
            bin_hdr.num_of_trcs_in_file
        );
        println!(
            "Byte offset of first trace: {}",
            bin_hdr.byte_off_of_first_trc
        );
        println!("Number of trailer stanzas: {}", bin_hdr.trailer_stanza_num);
    }

    #[test]
    fn test_trace_header_reading() {
        let mut sgy =
            Reader::<f32>::open("samples/ieee_single.sgy").expect("Problem opening the file");
        let hdrs = sgy
            .read_headers::<i32>(&[trc_hdr_names::TRC_SEQ_SEGY, trc_hdr_names::OFFSET])
            .expect("Error on headers reading");
        let mut ref_vec = Vec::new();
        let mut offset = 0;
        for j in 0..160 {
            if j % 40 == 0 {
                offset = 0;
            }
            ref_vec.push(vec![j + 1, offset]);
            offset += 50;
        }
        assert_eq!(hdrs, ref_vec);
    }
    #[test]
    fn test_single_trace_samples_reading() {
        let mut sgy =
            Reader::<f32>::open("samples/ieee_single.sgy").expect("Problem opening the file");
        let samples = sgy.read_samples_once().expect("Error on samples reading");
        assert_eq!(-2.1558735e-14, samples[samples.len() - 1]);
    }
    #[test]
    fn test_trace_samples_reading() {
        let mut sgy =
            Reader::<f32>::open("samples/ieee_single.sgy").expect("Problem opening the file");
        let samples = sgy.read_samples().expect("Error on samples reading");
        for i in 0..160 {
            assert_eq!(-2.1558735e-14, samples[i][samples[i].len() - 1 - i]);
        }
    }
    #[test]
    fn test_data_reading() {
        let mut sgy =
            Reader::<f32>::open("samples/ieee_single.sgy").expect("Problem opening the file");
        let hdr_names: Vec<i32> = (0..90).collect();
        let (headers, samples) = sgy
            .read_traces::<i32>(&hdr_names)
            .expect("Error on samples reading");
        let mut offset = 0;
        let mut chan = 1;
        let mut esp = 0;
        let mut sou_y = -50;
        let mut stat = 0;
        let mut ref_vec = Vec::new();
        for j in 0..160 {
            let mut tmp_vec = Vec::new();
            tmp_vec.push(j + 1);
            tmp_vec.push(j + 1);
            tmp_vec.push(0);
            if j % 40 == 0 {
                offset = 0;
                chan = 1;
                esp += 1;
                sou_y += 50;
            }
            tmp_vec.push(chan);
            tmp_vec.push(esp);
            tmp_vec.append(&mut vec![0; 2]);
            tmp_vec.push(1);
            tmp_vec.push(0);
            tmp_vec.push(1);
            tmp_vec.push(0);
            tmp_vec.push(offset);
            tmp_vec.append(&mut vec![0; 7]);
            tmp_vec.append(&mut vec![1; 2]);
            tmp_vec.push(0);
            tmp_vec.push(sou_y);
            tmp_vec.push(offset);
            tmp_vec.push(sou_y);
            tmp_vec.push(3);
            tmp_vec.append(&mut vec![0; 6]);
            tmp_vec.push(stat * 2);
            tmp_vec.append(&mut vec![0; 3]);
            tmp_vec.push(stat);
            tmp_vec.push(stat);
            tmp_vec.push(501);
            tmp_vec.push(2000);
            tmp_vec.append(&mut vec![0; 42]);
            tmp_vec.push(1);
            tmp_vec.append(&mut vec![0; 7]);
            ref_vec.push(tmp_vec);
            offset += 50;
            chan += 1;
            stat -= 2;
        }
        for i in 0..160 {
            assert_eq!(ref_vec, headers);
            assert_eq!(-2.1558735e-14, samples[i][samples[i].len() - 1 - i]);
        }
    }
    #[test]
    fn test_read_i8() {
        let arr = [0xffu8, 0xeeu8];
        let mut ptr = &arr[..];
        assert_eq!(read_i8(&mut ptr), 0xffu8 as i8);
        assert_eq!(read_i8(&mut ptr), 0xeeu8 as i8);
    }
    #[test]
    fn test_read_u8() {
        let arr = [0xffu8, 0xeeu8];
        let mut ptr = &arr[..];
        assert_eq!(read_u8(&mut ptr), 0xffu8);
        assert_eq!(read_u8(&mut ptr), 0xeeu8);
    }
    #[test]
    fn test_read_i16() {
        let arr = if cfg!(target_endian = "big") {
            [0xffu8, 0xeeu8, 0xddu8, 0xccu8]
        } else {
            [0xccu8, 0xddu8, 0xeeu8, 0xffu8]
        };
        let mut ptr = &arr[..];
        assert_eq!(read_i16(&mut ptr), 0xddccu16 as i16);
        assert_eq!(read_i16(&mut ptr), 0xffeeu16 as i16);
    }
    #[test]
    fn test_read_u16() {
        let arr = if cfg!(target_endian = "big") {
            [0xffu8, 0xeeu8, 0xddu8, 0xccu8]
        } else {
            [0xccu8, 0xddu8, 0xeeu8, 0xffu8]
        };
        let mut ptr = &arr[..];
        assert_eq!(read_u16(&mut ptr), 0xddccu16);
        assert_eq!(read_u16(&mut ptr), 0xffeeu16);
    }
    #[test]
    fn test_read_i24() {
        let arr = if cfg!(target_endian = "big") {
            [0xffu8, 0xeeu8, 0xddu8, 0xccu8, 0xbbu8, 0xaau8]
        } else {
            [0xaau8, 0xbbu8, 0xccu8, 0xddu8, 0xeeu8, 0xffu8]
        };
        let mut ptr = &arr[..];
        assert_eq!(read_i24(&mut ptr), 0xccbbaau32 as i32);
        assert_eq!(read_i24(&mut ptr), 0xffeeddu32 as i32);
    }
    #[test]
    fn test_read_u24() {
        let arr = if cfg!(target_endian = "big") {
            [0xffu8, 0xeeu8, 0xddu8, 0xccu8, 0xbbu8, 0xaau8]
        } else {
            [0xaau8, 0xbbu8, 0xccu8, 0xddu8, 0xeeu8, 0xffu8]
        };
        let mut ptr = &arr[..];
        assert_eq!(read_u24(&mut ptr), 0xccbbaau32);
        assert_eq!(read_u24(&mut ptr), 0xffeeddu32);
    }
    #[test]
    fn test_read_i32() {
        let arr = if cfg!(target_endian = "big") {
            [
                0xffu8, 0xeeu8, 0xddu8, 0xccu8, 0xbbu8, 0xaau8, 0x99u8, 0x88u8,
            ]
        } else {
            [
                0x88u8, 0x99u8, 0xaau8, 0xbbu8, 0xccu8, 0xddu8, 0xeeu8, 0xffu8,
            ]
        };
        let mut ptr = &arr[..];
        assert_eq!(read_i32(&mut ptr), 0xbbaa9988u32 as i32);
        assert_eq!(read_i32(&mut ptr), 0xffeeddccu32 as i32);
    }
    #[test]
    fn test_read_u32() {
        let arr = if cfg!(target_endian = "big") {
            [
                0xffu8, 0xeeu8, 0xddu8, 0xccu8, 0xbbu8, 0xaau8, 0x99u8, 0x88u8,
            ]
        } else {
            [
                0x88u8, 0x99u8, 0xaau8, 0xbbu8, 0xccu8, 0xddu8, 0xeeu8, 0xffu8,
            ]
        };
        let mut ptr = &arr[..];
        assert_eq!(read_u32(&mut ptr), 0xbbaa9988u32);
        assert_eq!(read_u32(&mut ptr), 0xffeeddccu32);
    }
    #[test]
    fn test_read_i64() {
        let arr = if cfg!(target_endian = "big") {
            [
                0xffu8, 0xeeu8, 0xddu8, 0xccu8, 0xbbu8, 0xaau8, 0x99u8, 0x88u8, 0x77u8, 0x66u8,
                0x55u8, 0x44u8, 0x33u8, 0x22u8, 0x11u8, 0x00u8,
            ]
        } else {
            [
                0x00u8, 0x11u8, 0x22u8, 0x33u8, 0x44u8, 0x55u8, 0x66u8, 0x77u8, 0x88u8, 0x99u8,
                0xaau8, 0xbbu8, 0xccu8, 0xddu8, 0xeeu8, 0xffu8,
            ]
        };
        let mut ptr = &arr[..];
        assert_eq!(read_i64(&mut ptr), 0x7766554433221100u64 as i64);
        assert_eq!(read_i64(&mut ptr), 0xffeeddccbbaa9988u64 as i64);
    }
    #[test]
    fn test_read_u64() {
        let arr = if cfg!(target_endian = "big") {
            [
                0xffu8, 0xeeu8, 0xddu8, 0xccu8, 0xbbu8, 0xaau8, 0x99u8, 0x88u8, 0x77u8, 0x66u8,
                0x55u8, 0x44u8, 0x33u8, 0x22u8, 0x11u8, 0x00u8,
            ]
        } else {
            [
                0x00u8, 0x11u8, 0x22u8, 0x33u8, 0x44u8, 0x55u8, 0x66u8, 0x77u8, 0x88u8, 0x99u8,
                0xaau8, 0xbbu8, 0xccu8, 0xddu8, 0xeeu8, 0xffu8,
            ]
        };
        let mut ptr = &arr[..];
        assert_eq!(read_u64(&mut ptr), 0x7766554433221100u64);
        assert_eq!(read_u64(&mut ptr), 0xffeeddccbbaa9988u64);
    }
    #[test]
    fn test_read_i16_sw() {
        let arr = if cfg!(target_endian = "big") {
            [0xffu8, 0xeeu8, 0xddu8, 0xccu8]
        } else {
            [0xccu8, 0xddu8, 0xeeu8, 0xffu8]
        };
        let mut ptr = &arr[..];
        assert_eq!(read_i16_sw(&mut ptr), 0xccddu16 as i16);
        assert_eq!(read_i16_sw(&mut ptr), 0xeeffu16 as i16);
    }
    #[test]
    fn test_read_u16_sw() {
        let arr = if cfg!(target_endian = "big") {
            [0xffu8, 0xeeu8, 0xddu8, 0xccu8]
        } else {
            [0xccu8, 0xddu8, 0xeeu8, 0xffu8]
        };
        let mut ptr = &arr[..];
        assert_eq!(read_u16_sw(&mut ptr), 0xccddu16);
        assert_eq!(read_u16_sw(&mut ptr), 0xeeffu16);
    }
    #[test]
    fn test_read_i24_sw() {
        let arr = if cfg!(target_endian = "big") {
            [0xffu8, 0xeeu8, 0xddu8, 0xccu8, 0xbbu8, 0xaau8]
        } else {
            [0xaau8, 0xbbu8, 0xccu8, 0xddu8, 0xeeu8, 0xffu8]
        };
        let mut ptr = &arr[..];
        assert_eq!(read_i24_sw(&mut ptr), 0xaabbccu32 as i32);
        assert_eq!(read_i24_sw(&mut ptr), 0xddeeffu32 as i32);
    }
    #[test]
    fn test_read_u24_sw() {
        let arr = if cfg!(target_endian = "big") {
            [0xffu8, 0xeeu8, 0xddu8, 0xccu8, 0xbbu8, 0xaau8]
        } else {
            [0xaau8, 0xbbu8, 0xccu8, 0xddu8, 0xeeu8, 0xffu8]
        };
        let mut ptr = &arr[..];
        assert_eq!(read_u24_sw(&mut ptr), 0xaabbccu32);
        assert_eq!(read_u24_sw(&mut ptr), 0xddeeffu32);
    }
    #[test]
    fn test_read_i32_sw() {
        let arr = if cfg!(target_endian = "big") {
            [
                0xffu8, 0xeeu8, 0xddu8, 0xccu8, 0xbbu8, 0xaau8, 0x99u8, 0x88u8,
            ]
        } else {
            [
                0x88u8, 0x99u8, 0xaau8, 0xbbu8, 0xccu8, 0xddu8, 0xeeu8, 0xffu8,
            ]
        };
        let mut ptr = &arr[..];
        assert_eq!(read_i32_sw(&mut ptr), 0x8899aabbu32 as i32);
        assert_eq!(read_i32_sw(&mut ptr), 0xccddeeffu32 as i32);
    }
    #[test]
    fn test_read_u32_sw() {
        let arr = if cfg!(target_endian = "big") {
            [
                0xffu8, 0xeeu8, 0xddu8, 0xccu8, 0xbbu8, 0xaau8, 0x99u8, 0x88u8,
            ]
        } else {
            [
                0x88u8, 0x99u8, 0xaau8, 0xbbu8, 0xccu8, 0xddu8, 0xeeu8, 0xffu8,
            ]
        };
        let mut ptr = &arr[..];
        assert_eq!(read_u32_sw(&mut ptr), 0x8899aabbu32);
        assert_eq!(read_u32_sw(&mut ptr), 0xccddeeffu32);
    }
    #[test]
    fn test_read_i64_sw() {
        let arr = if cfg!(target_endian = "big") {
            [
                0xffu8, 0xeeu8, 0xddu8, 0xccu8, 0xbbu8, 0xaau8, 0x99u8, 0x88u8, 0x77u8, 0x66u8,
                0x55u8, 0x44u8, 0x33u8, 0x22u8, 0x11u8, 0x00u8,
            ]
        } else {
            [
                0x00u8, 0x11u8, 0x22u8, 0x33u8, 0x44u8, 0x55u8, 0x66u8, 0x77u8, 0x88u8, 0x99u8,
                0xaau8, 0xbbu8, 0xccu8, 0xddu8, 0xeeu8, 0xffu8,
            ]
        };
        let mut ptr = &arr[..];
        assert_eq!(read_i64_sw(&mut ptr), 0x0011223344556677u64 as i64);
        assert_eq!(read_i64_sw(&mut ptr), 0x8899aabbccddeeffu64 as i64);
    }
    #[test]
    fn test_read_u64_sw() {
        let arr = if cfg!(target_endian = "big") {
            [
                0xffu8, 0xeeu8, 0xddu8, 0xccu8, 0xbbu8, 0xaau8, 0x99u8, 0x88u8, 0x77u8, 0x66u8,
                0x55u8, 0x44u8, 0x33u8, 0x22u8, 0x11u8, 0x00u8,
            ]
        } else {
            [
                0x00u8, 0x11u8, 0x22u8, 0x33u8, 0x44u8, 0x55u8, 0x66u8, 0x77u8, 0x88u8, 0x99u8,
                0xaau8, 0xbbu8, 0xccu8, 0xddu8, 0xeeu8, 0xffu8,
            ]
        };
        let mut ptr = &arr[..];
        assert_eq!(read_u64_sw(&mut ptr), 0x0011223344556677u64);
        assert_eq!(read_u64_sw(&mut ptr), 0x8899aabbccddeeffu64);
    }
}
