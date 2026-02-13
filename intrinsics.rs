use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "C" fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void {
    let dest_u8 = dest as *mut u8;
    let src_u8 = src as *const u8;
    for i in 0..n {
        *dest_u8.add(i) = *src_u8.add(i);
    }
    dest
}

#[no_mangle]
pub unsafe extern "C" fn memset(s: *mut c_void, c: i32, n: usize) -> *mut c_void {
    let s_u8 = s as *mut u8;
    let value = c as u8;
    for i in 0..n {
        *s_u8.add(i) = value;
    }
    s
}

#[no_mangle]
pub unsafe extern "C" fn memcmp(s1: *const c_void, s2: *const c_void, n: usize) -> i32 {
    let s1_u8 = s1 as *const u8;
    let s2_u8 = s2 as *const u8;
    for i in 0..n {
        let a = *s1_u8.add(i);
        let b = *s2_u8.add(i);
        if a != b {
            return a as i32 - b as i32;
        }
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn bcmp(s1: *const c_void, s2: *const c_void, n: usize) -> i32 {
    memcmp(s1, s2, n)
}

#[no_mangle]
pub unsafe extern "C" fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void {
    let dest_u8 = dest as *mut u8;
    let src_u8 = src as *const u8;
    
    if (dest_u8 as usize) < (src_u8 as usize) {
        for i in 0..n {
            *dest_u8.add(i) = *src_u8.add(i);
        }
    } else {
        for i in (0..n).rev() {
            *dest_u8.add(i) = *src_u8.add(i);
        }
    }
    dest
}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}
