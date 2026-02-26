use crate::common::{
    BIN_HEADER_SIZE, BinaryHeader, Primitive, TEXT_HEADER_SIZE, TRACE_HEADER_SIZE, TrcHdrFmt,
    std_trc_hdr_map,
};
use crate::error::Error;
use std::fs::File;
use std::io::Write;

macro_rules! write {
    ($buf:expr, $value:expr) => {{
        let size = std::mem::size_of_val(&$value);
        let bytes = $value.to_ne_bytes();
        $buf[..size].copy_from_slice(&bytes);
        &mut $buf[size..]
    }};
}

pub struct Writer<T> {
    file: File,
    hdr_buf: Vec<u8>,
    samp_buf: Vec<u8>,
    wrt_fun: WrtFun,
    write_one_sample: for<'a, 'b> fn(&'a WrtFun, &'b mut [u8], T) -> &'b mut [u8],
    bytes_per_sample: usize,
    max_add_trc_hdrs: i32,
}

struct WrtFun {
    write_u64: fn(&mut [u8], u64) -> &mut [u8],
    write_u32: fn(&mut [u8], u32) -> &mut [u8],
    write_u24: fn(&mut [u8], u32) -> &mut [u8],
    write_u16: fn(&mut [u8], u16) -> &mut [u8],
    write_u8: fn(&mut [u8], u8) -> &mut [u8],
    write_i64: fn(&mut [u8], i64) -> &mut [u8],
    write_i32: fn(&mut [u8], i32) -> &mut [u8],
    write_i24: fn(&mut [u8], i32) -> &mut [u8],
    write_i16: fn(&mut [u8], i16) -> &mut [u8],
    write_i8: fn(&mut [u8], i8) -> &mut [u8],
}

impl<T: Primitive + Clone + Copy> Writer<T> {
    pub fn create(
        path: &str,
        txt_hdr: &[u8; TEXT_HEADER_SIZE],
        bin_hdr: BinaryHeader,
    ) -> Result<Self, Error> {
        check_bin_hdr(&bin_hdr)?;
        let mut file = File::create(path)?;
        file.write_all(txt_hdr)?;
        let (
            write_u64,
            write_u32,
            write_u24,
            write_u16,
            write_i64,
            write_i32,
            write_i24,
            write_i16,
        ): (
            fn(&mut [u8], u64) -> &mut [u8],
            fn(&mut [u8], u32) -> &mut [u8],
            fn(&mut [u8], u32) -> &mut [u8],
            fn(&mut [u8], u16) -> &mut [u8],
            fn(&mut [u8], i64) -> &mut [u8],
            fn(&mut [u8], i32) -> &mut [u8],
            fn(&mut [u8], i32) -> &mut [u8],
            fn(&mut [u8], i16) -> &mut [u8],
        ) = match bin_hdr.endianness {
            0x01020304 => (
                write_u64, write_u32, write_u24, write_u16, write_i64, write_i32, write_i24,
                write_i16,
            ),
            0x04030201 | 0 => (
                write_u64_sw,
                write_u32_sw,
                write_u24_sw,
                write_u16_sw,
                write_i64_sw,
                write_i32_sw,
                write_i24_sw,
                write_i16_sw,
            ),
            _ => return Err(Error::UnsupportedEndianness(bin_hdr.endianness as u32)),
        };
        let write_u8 = write_u8;
        let write_i8 = write_i8;
        let mut bin_buf = [0u8; BIN_HEADER_SIZE];
        let mut ptr = &mut bin_buf[..];
        ptr = write_i32(ptr, bin_hdr.job_id);
        ptr = write_i32(ptr, bin_hdr.line_num);
        ptr = write_i32(ptr, bin_hdr.reel_num);
        ptr = write_i16(ptr, bin_hdr.trc_num);
        ptr = write_i16(ptr, bin_hdr.aux_trc_num);
        ptr = write_i16(ptr, bin_hdr.samp_int);
        ptr = write_i16(ptr, bin_hdr.samp_int_orig);
        ptr = write_i16(ptr, bin_hdr.samp_num);
        ptr = write_i16(ptr, bin_hdr.samp_num_orig);
        ptr = write_i16(ptr, bin_hdr.format_code);
        ptr = write_i16(ptr, bin_hdr.ensemble_fold);
        ptr = write_i16(ptr, bin_hdr.trc_sort_code);
        ptr = write_i16(ptr, bin_hdr.vert_sum_code);
        ptr = write_i16(ptr, bin_hdr.sw_freq_at_start);
        ptr = write_i16(ptr, bin_hdr.sw_freq_at_end);
        ptr = write_i16(ptr, bin_hdr.sw_length);
        ptr = write_i16(ptr, bin_hdr.sw_type_code);
        ptr = write_i16(ptr, bin_hdr.trc_num_of_sw_ch);
        ptr = write_i16(ptr, bin_hdr.sw_trc_taper_length_start);
        ptr = write_i16(ptr, bin_hdr.sw_trc_taper_length_end);
        ptr = write_i16(ptr, bin_hdr.taper_type);
        ptr = write_i16(ptr, bin_hdr.corr_data_trc);
        ptr = write_i16(ptr, bin_hdr.bin_gain_recov);
        ptr = write_i16(ptr, bin_hdr.amp_recov_meth);
        ptr = write_i16(ptr, bin_hdr.measure_sys);
        ptr = write_i16(ptr, bin_hdr.impulse_sig_pol);
        ptr = write_i16(ptr, bin_hdr.vib_pol_code);
        ptr = write_i32(ptr, bin_hdr.ext_trc_num);
        ptr = write_i32(ptr, bin_hdr.ext_aux_trc_num);
        ptr = write_i32(ptr, bin_hdr.ext_samp_num);
        ptr = write_u64(ptr, bin_hdr.ext_samp_int.to_bits());
        ptr = write_u64(ptr, bin_hdr.ext_samp_int_orig.to_bits());
        ptr = write_i32(ptr, bin_hdr.ext_samp_num_orig);
        ptr = write_i32(ptr, bin_hdr.ext_ensemble_fold);
        write_i32(ptr, bin_hdr.endianness);
        ptr = &mut bin_buf[300..];
        ptr = write_u8(ptr, bin_hdr.segy_maj_ver);
        ptr = write_u8(ptr, bin_hdr.segy_min_ver);
        ptr = write_i16(ptr, bin_hdr.fix_length_trc_flag);
        ptr = write_i16(ptr, bin_hdr.ext_text_hdrs_num);
        ptr = write_i32(ptr, bin_hdr.max_add_trc_hdrs);
        ptr = write_i16(ptr, bin_hdr.time_basis_code);
        ptr = write_u64(ptr, bin_hdr.num_of_trcs_in_file);
        ptr = write_u64(ptr, bin_hdr.byte_off_of_first_trc);
        write_i32(ptr, bin_hdr.trailer_stanza_num);
        file.write_all(&bin_buf)?;
        let write_one_sample = match bin_hdr.format_code {
            1 => Writer::sample_as_ibm,
            2 => Writer::sample_as_i32,
            3 => Writer::sample_as_i16,
            5 => Writer::sample_as_f32,
            6 => Writer::sample_as_f64,
            7 => Writer::sample_as_i24,
            8 => Writer::sample_as_i8,
            9 => Writer::sample_as_i64,
            10 => Writer::sample_as_u32,
            11 => Writer::sample_as_u16,
            12 => Writer::sample_as_u64,
            15 => Writer::sample_as_u24,
            16 => Writer::sample_as_u8,
            _ => return Err(Error::UnsupportedFormatCode(bin_hdr.format_code)),
        };
        let hdr_buf = vec![0u8; TRACE_HEADER_SIZE * (bin_hdr.max_add_trc_hdrs + 1) as usize];
        let bytes_per_sample: usize = match bin_hdr.format_code {
            8 | 16 => 1,
            3 | 11 => 2,
            7 | 15 => 3,
            1 | 2 | 5 | 10 => 4,
            6 | 9 | 12 => 8,
            _ => return Err(Error::UnsupportedFormatCode(bin_hdr.format_code)),
        };
        let samp_num = if bin_hdr.ext_samp_num != 0 {
            bin_hdr.ext_samp_num
        } else {
            bin_hdr.samp_num as i32
        };
        let samp_buf = vec![0u8; samp_num as usize * bytes_per_sample];
        Ok(Writer {
            file,
            hdr_buf,
            samp_buf,
            wrt_fun: WrtFun {
                write_u64,
                write_u32,
                write_u24,
                write_u16,
                write_u8,
                write_i64,
                write_i32,
                write_i24,
                write_i16,
                write_i8,
            },
            write_one_sample,
            bytes_per_sample,
            max_add_trc_hdrs: bin_hdr.max_add_trc_hdrs,
        })
    }
    pub fn close(self) {}
    pub fn write_one_trace<U: Primitive + Copy>(
        &mut self,
        hdr_names: &Vec<i32>,
        hdr_vals: &Vec<U>,
        samples: &Vec<T>,
    ) -> Result<(), Error> {
        let hdr_map = std_trc_hdr_map();
        for (i, hn) in hdr_names.iter().enumerate() {
            let (fmt, offset) = hdr_map[hn];
            let offset = offset as usize;
            let ptr = &mut self.hdr_buf[offset..];
            _ = match fmt {
                TrcHdrFmt::I8 => (self.wrt_fun.write_i8)(ptr, U::as_i8(hdr_vals[i])),
                TrcHdrFmt::I16 => (self.wrt_fun.write_i16)(ptr, U::as_i16(hdr_vals[i])),
                TrcHdrFmt::I32 => (self.wrt_fun.write_i32)(ptr, U::as_i32(hdr_vals[i])),
                TrcHdrFmt::I64 => (self.wrt_fun.write_i64)(ptr, U::as_i64(hdr_vals[i])),
                TrcHdrFmt::U8 => (self.wrt_fun.write_u8)(ptr, U::as_u8(hdr_vals[i])),
                TrcHdrFmt::U16 => (self.wrt_fun.write_u16)(ptr, U::as_u16(hdr_vals[i])),
                TrcHdrFmt::U32 => (self.wrt_fun.write_u32)(ptr, U::as_u32(hdr_vals[i])),
                TrcHdrFmt::U64 => (self.wrt_fun.write_u64)(ptr, U::as_u64(hdr_vals[i])),
                TrcHdrFmt::F32 => (self.wrt_fun.write_u32)(ptr, U::as_f32(hdr_vals[i]).to_bits()),
                TrcHdrFmt::F64 => (self.wrt_fun.write_u64)(ptr, U::as_f64(hdr_vals[i]).to_bits()),
            };
        }
        self.file.write_all(&self.hdr_buf)?;
        if self.samp_buf.len() / 4 != samples.len() {
            self.samp_buf
                .resize(samples.len() * self.bytes_per_sample, 0u8);
        }
        let mut ptr = &mut self.samp_buf[..];
        for samp in samples {
            ptr = (self.write_one_sample)(&self.wrt_fun, ptr, *samp);
        }
        Ok(())
    }
    pub fn write_traces<U: Primitive + Copy>(
        &mut self,
        hdr_names: &[i32],
        data: (&Vec<Vec<U>>, &Vec<Vec<T>>),
    ) -> Result<(), Error> {
        if data.0.len() != data.1.len() || data.0[0].len() != hdr_names.len() {
            return Err(Error::DiffDimToWrite());
        }
        let hdr_map = std_trc_hdr_map();
        for i in 0..hdr_names.len() {
            let (hdr_fmt, offset) = hdr_map[&hdr_names[i]];
            let format_size = match hdr_fmt {
                TrcHdrFmt::I8 | TrcHdrFmt::U8 => 1,
                TrcHdrFmt::I16 | TrcHdrFmt::U16 => 2,
                TrcHdrFmt::I32 | TrcHdrFmt::U32 | TrcHdrFmt::F32 => 4,
                TrcHdrFmt::I64 | TrcHdrFmt::U64 | TrcHdrFmt::F64 => 8,
            };
            if offset + format_size > (self.max_add_trc_hdrs + 1) * TRACE_HEADER_SIZE as i32 {
                return Err(Error::TraceHeaderMap(i as i32));
            }
        }
        let hdr_vals = &data.0;
        let samples = &data.1;
        for i in 0..hdr_vals.len() {
            for (j, hn) in hdr_names.iter().enumerate() {
                let (fmt, offset) = hdr_map[hn];
                let offset = offset as usize;
                let ptr = &mut self.hdr_buf[offset..];
                _ = match fmt {
                    TrcHdrFmt::I8 => (self.wrt_fun.write_i8)(ptr, U::as_i8(hdr_vals[i][j])),
                    TrcHdrFmt::I16 => (self.wrt_fun.write_i16)(ptr, U::as_i16(hdr_vals[i][j])),
                    TrcHdrFmt::I32 => (self.wrt_fun.write_i32)(ptr, U::as_i32(hdr_vals[i][j])),
                    TrcHdrFmt::I64 => (self.wrt_fun.write_i64)(ptr, U::as_i64(hdr_vals[i][j])),
                    TrcHdrFmt::U8 => (self.wrt_fun.write_u8)(ptr, U::as_u8(hdr_vals[i][j])),
                    TrcHdrFmt::U16 => (self.wrt_fun.write_u16)(ptr, U::as_u16(hdr_vals[i][j])),
                    TrcHdrFmt::U32 => (self.wrt_fun.write_u32)(ptr, U::as_u32(hdr_vals[i][j])),
                    TrcHdrFmt::U64 => (self.wrt_fun.write_u64)(ptr, U::as_u64(hdr_vals[i][j])),
                    TrcHdrFmt::F32 => {
                        (self.wrt_fun.write_u32)(ptr, U::as_f32(hdr_vals[i][j]).to_bits())
                    }
                    TrcHdrFmt::F64 => {
                        (self.wrt_fun.write_u64)(ptr, U::as_f64(hdr_vals[i][j]).to_bits())
                    }
                };
            }
            self.file.write_all(&self.hdr_buf)?;
            let mut ptr = &mut self.samp_buf[..];
            for samp in &samples[i] {
                ptr = (self.write_one_sample)(&self.wrt_fun, ptr, *samp);
            }
            self.file.write_all(&self.samp_buf)?;
        }
        Ok(())
    }
    fn sample_as_ibm<'a, 'b>(wrt_fun: &'a WrtFun, buf: &'b mut [u8], val: T) -> &'b mut [u8] {
        let val = T::as_f64(val);
        let sign: u32 = if val < 0.0 { 1 } else { 0 };
        let val = val.abs();
        let exp = (val.ln() / 2f64.ln() / 4.0 + 1.0 + 64.0) as u32 & 0x7f;
        let fraction = (val / 16f64.powi(exp as i32 - 64) * 2u32.pow(24) as f64) as u32;
        let result = sign << 31 | exp << 24 | (fraction & 0x00ffffff);
        (wrt_fun.write_u32)(buf, result)
    }
    fn sample_as_f32<'a, 'b>(wrt_fun: &'a WrtFun, buf: &'b mut [u8], val: T) -> &'b mut [u8] {
        (wrt_fun.write_u32)(buf, T::as_f32(val).to_bits())
    }
    fn sample_as_f64<'a, 'b>(wrt_fun: &'a WrtFun, buf: &'b mut [u8], val: T) -> &'b mut [u8] {
        (wrt_fun.write_u64)(buf, T::as_f64(val).to_bits())
    }
    fn sample_as_u64<'a, 'b>(wrt_fun: &'a WrtFun, buf: &'b mut [u8], val: T) -> &'b mut [u8] {
        (wrt_fun.write_u64)(buf, T::as_u64(val))
    }
    fn sample_as_i64<'a, 'b>(wrt_fun: &'a WrtFun, buf: &'b mut [u8], val: T) -> &'b mut [u8] {
        (wrt_fun.write_i64)(buf, T::as_i64(val))
    }
    fn sample_as_u32<'a, 'b>(wrt_fun: &'a WrtFun, buf: &'b mut [u8], val: T) -> &'b mut [u8] {
        (wrt_fun.write_u32)(buf, T::as_u32(val))
    }
    fn sample_as_i32<'a, 'b>(wrt_fun: &'a WrtFun, buf: &'b mut [u8], val: T) -> &'b mut [u8] {
        (wrt_fun.write_i32)(buf, T::as_i32(val))
    }
    fn sample_as_u24<'a, 'b>(wrt_fun: &'a WrtFun, buf: &'b mut [u8], val: T) -> &'b mut [u8] {
        (wrt_fun.write_u24)(buf, T::as_u32(val))
    }
    fn sample_as_i24<'a, 'b>(wrt_fun: &'a WrtFun, buf: &'b mut [u8], val: T) -> &'b mut [u8] {
        (wrt_fun.write_i24)(buf, T::as_i32(val))
    }
    fn sample_as_u16<'a, 'b>(wrt_fun: &'a WrtFun, buf: &'b mut [u8], val: T) -> &'b mut [u8] {
        (wrt_fun.write_u16)(buf, T::as_u16(val))
    }
    fn sample_as_i16<'a, 'b>(wrt_fun: &'a WrtFun, buf: &'b mut [u8], val: T) -> &'b mut [u8] {
        (wrt_fun.write_i16)(buf, T::as_i16(val))
    }
    fn sample_as_u8<'a, 'b>(wrt_fun: &'a WrtFun, buf: &'b mut [u8], val: T) -> &'b mut [u8] {
        (wrt_fun.write_u8)(buf, T::as_u8(val))
    }
    fn sample_as_i8<'a, 'b>(wrt_fun: &'a WrtFun, buf: &'b mut [u8], val: T) -> &'b mut [u8] {
        (wrt_fun.write_i8)(buf, T::as_i8(val))
    }
}

fn check_bin_hdr(bh: &BinaryHeader) -> Result<(), Error> {
    if bh.samp_int == 0 && bh.ext_samp_int == 0.0 {
        return Err(Error::ZeroSampleInterval());
    }
    if bh.samp_num == 0 && bh.ext_samp_num == 0 {
        return Err(Error::ZeroSampleNumber());
    }
    match bh.segy_maj_ver {
        0 => {
            if bh.format_code < 1 || bh.format_code > 3 {
                return Err(Error::WrongFormatForRevision(bh.format_code));
            }
        }
        1 => {
            if bh.format_code < 1
                || bh.format_code == 3
                || bh.format_code == 6
                || bh.format_code == 7
                || bh.format_code > 8
            {
                return Err(Error::WrongFormatForRevision(bh.format_code));
            }
        }
        2 => {
            if bh.format_code < 1
                || bh.format_code == 3
                || bh.format_code == 13
                || bh.format_code == 14
                || bh.format_code > 16
            {
                return Err(Error::WrongFormatForRevision(bh.format_code));
            }
        }
        _ => return Err(Error::UnsupportedRevision(bh.segy_maj_ver)),
    }
    Ok(())
}

fn write_i8(buf: &mut [u8], val: i8) -> &mut [u8] {
    write!(buf, val)
}
fn write_u8(buf: &mut [u8], val: u8) -> &mut [u8] {
    write!(buf, val)
}
fn write_i16(buf: &mut [u8], val: i16) -> &mut [u8] {
    write!(buf, val)
}
fn write_u16(buf: &mut [u8], val: u16) -> &mut [u8] {
    write!(buf, val)
}
fn write_i24(buf: &mut [u8], val: i32) -> &mut [u8] {
    let tmp = write!(buf, (val & 0xffff) as u16);
    write!(tmp, ((val & 0xff0000) >> 16) as u8)
}
fn write_u24(buf: &mut [u8], val: u32) -> &mut [u8] {
    let tmp = write!(buf, (val & 0xffff) as u16);
    write!(tmp, ((val & 0xff0000) >> 16) as u8)
}
fn write_i32(buf: &mut [u8], val: i32) -> &mut [u8] {
    write!(buf, val)
}
fn write_u32(buf: &mut [u8], val: u32) -> &mut [u8] {
    write!(buf, val)
}
fn write_i64(buf: &mut [u8], val: i64) -> &mut [u8] {
    write!(buf, val)
}
fn write_u64(buf: &mut [u8], val: u64) -> &mut [u8] {
    write!(buf, val)
}
fn write_i16_sw(buf: &mut [u8], val: i16) -> &mut [u8] {
    write!(buf, val.swap_bytes())
}
fn write_u16_sw(buf: &mut [u8], val: u16) -> &mut [u8] {
    write!(buf, val.swap_bytes())
}
fn write_i24_sw(buf: &mut [u8], val: i32) -> &mut [u8] {
    let tmp = val.swap_bytes() >> 8;
    let tmp_buf = write!(buf, (tmp & 0xffff) as u16);
    write!(tmp_buf, ((tmp & 0xff0000) >> 16) as u8)
}
fn write_u24_sw(buf: &mut [u8], val: u32) -> &mut [u8] {
    let tmp = val.swap_bytes() >> 8;
    let tmp_buf = write!(buf, (tmp & 0xffff) as u16);
    write!(tmp_buf, ((tmp & 0xff0000) >> 16) as u8)
}
fn write_i32_sw(buf: &mut [u8], val: i32) -> &mut [u8] {
    write!(buf, val.swap_bytes())
}
fn write_u32_sw(buf: &mut [u8], val: u32) -> &mut [u8] {
    write!(buf, val.swap_bytes())
}
fn write_i64_sw(buf: &mut [u8], val: i64) -> &mut [u8] {
    write!(buf, val.swap_bytes())
}
fn write_u64_sw(buf: &mut [u8], val: u64) -> &mut [u8] {
    write!(buf, val.swap_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_traces() {
        let mut isgy = crate::reader::Reader::<f32>::open("samples/ieee_single.sgy")
            .expect("Problem opening file for reading");
        let txt_hdr = isgy
            .read_raw_text_header()
            .expect("Error on text header reading");
        let hdr_names: Vec<i32> = (0..90).collect();
        let (headers, samples) = isgy
            .read_traces::<i32>(&hdr_names)
            .expect("Error on trace reading");
        let mut osgy = Writer::<f32>::create(
            "samples/test_trc_wrt.sgy",
            &txt_hdr,
            isgy.get_binary_header().clone(),
        )
        .expect("Problem creating file for writing");
        osgy.write_traces(&hdr_names, (&headers, &samples))
            .expect("Error on traces writing");
        osgy.close();
        isgy.close();
        let ref_file = std::fs::read("samples/ieee_single.sgy")
            .expect("Error on reference file reading while check.");
        let created = std::fs::read("samples/test_trc_wrt.sgy")
            .expect("Error on created file reading while check.");
        assert_eq!(ref_file, created);
        std::fs::remove_file("samples/test_trc_wrt.sgy").expect("Error on file deleting");
    }
    #[test]
    fn test_write_i8() {
        let ref_arr = [0xffu8, 0xeeu8];
        let mut buf = [0u8; 2];
        let mut ptr = &mut buf[..];
        ptr = write_i8(ptr, 0xffu8 as i8);
        write_i8(ptr, 0xeeu8 as i8);
        assert_eq!(buf, ref_arr);
    }
    #[test]
    fn test_write_u8() {
        let ref_arr = [0xffu8, 0xeeu8];
        let mut buf = [0u8; 2];
        let mut ptr = &mut buf[..];
        ptr = write_u8(ptr, 0xffu8);
        write_u8(ptr, 0xeeu8);
        assert_eq!(buf, ref_arr);
    }
    #[test]
    fn test_write_i16() {
        let ref_arr = if cfg!(target_endian = "big") {
            [0xffu8, 0xeeu8, 0xddu8, 0xccu8]
        } else {
            [0xccu8, 0xddu8, 0xeeu8, 0xffu8]
        };
        let mut buf = [0u8; 4];
        let mut ptr = &mut buf[..];
        ptr = write_i16(ptr, 0xddccu16 as i16);
        write_i16(ptr, 0xffeeu16 as i16);
        assert_eq!(buf, ref_arr);
    }
    #[test]
    fn test_write_u16() {
        let ref_arr = if cfg!(target_endian = "big") {
            [0xffu8, 0xeeu8, 0xddu8, 0xccu8]
        } else {
            [0xccu8, 0xddu8, 0xeeu8, 0xffu8]
        };
        let mut buf = [0u8; 4];
        let mut ptr = &mut buf[..];
        ptr = write_u16(ptr, 0xddccu16);
        write_u16(ptr, 0xffeeu16);
        assert_eq!(buf, ref_arr);
    }
    #[test]
    fn test_write_i24() {
        let ref_arr = if cfg!(target_endian = "big") {
            [0xffu8, 0xeeu8, 0xddu8, 0xccu8, 0xbbu8, 0xaau8]
        } else {
            [0xaau8, 0xbbu8, 0xccu8, 0xddu8, 0xeeu8, 0xffu8]
        };
        let mut buf = [0u8; 6];
        let mut ptr = &mut buf[..];
        ptr = write_i24(ptr, 0xccbbaau32 as i32);
        write_i24(ptr, 0xffeeddu32 as i32);
        assert_eq!(buf, ref_arr);
    }
    #[test]
    fn test_write_u24() {
        let ref_arr = if cfg!(target_endian = "big") {
            [0xffu8, 0xeeu8, 0xddu8, 0xccu8, 0xbbu8, 0xaau8]
        } else {
            [0xaau8, 0xbbu8, 0xccu8, 0xddu8, 0xeeu8, 0xffu8]
        };
        let mut buf = [0u8; 6];
        let mut ptr = &mut buf[..];
        ptr = write_u24(ptr, 0xccbbaau32);
        write_u24(ptr, 0xffeeddu32);
        assert_eq!(buf, ref_arr);
    }
    #[test]
    fn test_write_i32() {
        let ref_arr = if cfg!(target_endian = "big") {
            [
                0xffu8, 0xeeu8, 0xddu8, 0xccu8, 0xbbu8, 0xaau8, 0x99u8, 0x88u8,
            ]
        } else {
            [
                0x88u8, 0x99u8, 0xaau8, 0xbbu8, 0xccu8, 0xddu8, 0xeeu8, 0xffu8,
            ]
        };
        let mut buf = [0u8; 8];
        let mut ptr = &mut buf[..];
        ptr = write_i32(ptr, 0xbbaa9988u32 as i32);
        write_i32(ptr, 0xffeeddccu32 as i32);
        assert_eq!(buf, ref_arr);
    }
    #[test]
    fn test_write_u32() {
        let ref_arr = if cfg!(target_endian = "big") {
            [
                0xffu8, 0xeeu8, 0xddu8, 0xccu8, 0xbbu8, 0xaau8, 0x99u8, 0x88u8,
            ]
        } else {
            [
                0x88u8, 0x99u8, 0xaau8, 0xbbu8, 0xccu8, 0xddu8, 0xeeu8, 0xffu8,
            ]
        };
        let mut buf = [0u8; 8];
        let mut ptr = &mut buf[..];
        ptr = write_u32(ptr, 0xbbaa9988u32);
        write_u32(ptr, 0xffeeddccu32);
        assert_eq!(buf, ref_arr);
    }
    #[test]
    fn test_write_i64() {
        let ref_arr = if cfg!(target_endian = "big") {
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
        let mut buf = [0u8; 16];
        let mut ptr = &mut buf[..];
        ptr = write_i64(ptr, 0x7766554433221100u64 as i64);
        write_i64(ptr, 0xffeeddccbbaa9988u64 as i64);
        assert_eq!(buf, ref_arr);
    }
    #[test]
    fn test_write_u64() {
        let ref_arr = if cfg!(target_endian = "big") {
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
        let mut buf = [0u8; 16];
        let mut ptr = &mut buf[..];
        ptr = write_u64(ptr, 0x7766554433221100u64);
        write_u64(ptr, 0xffeeddccbbaa9988u64);
        assert_eq!(buf, ref_arr);
    }
    #[test]
    fn test_write_i16_sw() {
        let ref_arr = if cfg!(target_endian = "big") {
            [0xffu8, 0xeeu8, 0xddu8, 0xccu8]
        } else {
            [0xccu8, 0xddu8, 0xeeu8, 0xffu8]
        };
        let mut buf = [0u8; 4];
        let mut ptr = &mut buf[..];
        ptr = write_i16_sw(ptr, 0xccddu16 as i16);
        write_i16_sw(ptr, 0xeeffu16 as i16);
        assert_eq!(buf, ref_arr);
    }
    #[test]
    fn test_write_u16_sw() {
        let ref_arr = if cfg!(target_endian = "big") {
            [0xffu8, 0xeeu8, 0xddu8, 0xccu8]
        } else {
            [0xccu8, 0xddu8, 0xeeu8, 0xffu8]
        };
        let mut buf = [0u8; 4];
        let mut ptr = &mut buf[..];
        ptr = write_u16_sw(ptr, 0xccddu16);
        write_u16_sw(ptr, 0xeeffu16);
        assert_eq!(buf, ref_arr);
    }
    #[test]
    fn test_write_i24_sw() {
        let ref_arr = if cfg!(target_endian = "big") {
            [0xffu8, 0xeeu8, 0xddu8, 0xccu8, 0xbbu8, 0xaau8]
        } else {
            [0xaau8, 0xbbu8, 0xccu8, 0xddu8, 0xeeu8, 0xffu8]
        };
        let mut buf = [0u8; 6];
        let mut ptr = &mut buf[..];
        ptr = write_i24_sw(ptr, 0xaabbccu32 as i32);
        write_i24_sw(ptr, 0xddeeffu32 as i32);
        assert_eq!(buf, ref_arr);
    }
    #[test]
    fn test_write_u24_sw() {
        let ref_arr = if cfg!(target_endian = "big") {
            [0xffu8, 0xeeu8, 0xddu8, 0xccu8, 0xbbu8, 0xaau8]
        } else {
            [0xaau8, 0xbbu8, 0xccu8, 0xddu8, 0xeeu8, 0xffu8]
        };
        let mut buf = [0u8; 6];
        let mut ptr = &mut buf[..];
        ptr = write_u24_sw(ptr, 0xaabbccu32);
        write_u24_sw(ptr, 0xddeeffu32);
        assert_eq!(buf, ref_arr);
    }
    #[test]
    fn test_write_i32_sw() {
        let ref_arr = if cfg!(target_endian = "big") {
            [
                0xffu8, 0xeeu8, 0xddu8, 0xccu8, 0xbbu8, 0xaau8, 0x99u8, 0x88u8,
            ]
        } else {
            [
                0x88u8, 0x99u8, 0xaau8, 0xbbu8, 0xccu8, 0xddu8, 0xeeu8, 0xffu8,
            ]
        };
        let mut buf = [0u8; 8];
        let mut ptr = &mut buf[..];
        ptr = write_i32_sw(ptr, 0x8899aabbu32 as i32);
        write_i32_sw(ptr, 0xccddeeffu32 as i32);
        assert_eq!(buf, ref_arr);
    }
    #[test]
    fn test_write_u32_sw() {
        let ref_arr = if cfg!(target_endian = "big") {
            [
                0xffu8, 0xeeu8, 0xddu8, 0xccu8, 0xbbu8, 0xaau8, 0x99u8, 0x88u8,
            ]
        } else {
            [
                0x88u8, 0x99u8, 0xaau8, 0xbbu8, 0xccu8, 0xddu8, 0xeeu8, 0xffu8,
            ]
        };
        let mut buf = [0u8; 8];
        let mut ptr = &mut buf[..];
        ptr = write_u32_sw(ptr, 0x8899aabbu32);
        write_u32_sw(ptr, 0xccddeeffu32);
        assert_eq!(buf, ref_arr);
    }
    #[test]
    fn test_write_i64_sw() {
        let ref_arr = if cfg!(target_endian = "big") {
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
        let mut buf = [0u8; 16];
        let mut ptr = &mut buf[..];
        ptr = write_i64_sw(ptr, 0x0011223344556677u64 as i64);
        write_i64_sw(ptr, 0x8899aabbccddeeffu64 as i64);
        assert_eq!(buf, ref_arr);
    }
    #[test]
    fn test_write_u64_sw() {
        let ref_arr = if cfg!(target_endian = "big") {
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
        let mut buf = [0u8; 16];
        let mut ptr = &mut buf[..];
        ptr = write_u64_sw(ptr, 0x0011223344556677u64);
        write_u64_sw(ptr, 0x8899aabbccddeeffu64);
        assert_eq!(buf, ref_arr);
    }
}
