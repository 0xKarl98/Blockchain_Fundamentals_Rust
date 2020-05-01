
use std::env;

/// The size of the blocks (chunks of data) our hash function
const BLOCK_SIZE: usize = 8;

/// The initialization vector (IV) of the BillHash function.
const INITIALIZATION_VECTOR: u64 = 0x0123_4567_89AB_CDEF;


/// Simple function to tell the user about appropriate usage and exit with exit code 1.
fn print_usage_and_exit() {
    println!("Please enter one and only one argument");
    std::process::exit(1);
}

/// Given a string, convert it to a vector of u8s
fn convert_string_to_u8s(s: String) -> Vec<u8> {
    s.as_bytes().to_vec()
}

/// "Wrapper function" which returns the string to hash, getting it from the
/// command line arguments.  It also does some simple housekeeping to ensure
/// that a single argument was passed in, and exits if not.

fn get_to_hash() -> String {
    let mut args = Vec::new();
    for argument in env::args() {
        args.push(argument);
    }

    // ignore "0 arg", i.e. the executable name itself
    let args_len = args.len() - 1;

    if args_len != 1 {
        print_usage_and_exit();
    }

    args[1].clone()

}

/// Strengthening can be thought of simply padding the end of the input string
/// with 0's so that it can be split up into equal blocks all of size BLOCK_SIZE.
/// Running the compress function will always involve passing in an array of
/// size BLOCK_SIZE, so we must ensure that we can do that now.  We simply
/// add 0s until data % BLOCK_SIZE == 0 and data.len() > 0.
/// Note that there is an edge where an empty vector is passed in.  In this case,
/// we will have to add BLOCK_SIZE number of 0's.
fn strengthen(data: Vec<u8>) -> Vec<u8> {
    let rem = data.len() % BLOCK_SIZE;
    if rem == 0 && data.len() > 0 {
        // do nothing, no padding necessary
        data
    } else {
        let mut to_return = data;
        let n = BLOCK_SIZE - rem;
        for _j in 0..n {
            to_return.push(0);
        }
        to_return
    }
}

/// The twiddle method "twiddles" the bits of the input array `arr` by XORing the
/// values of every other element in the array with itself, with different sized
/// left shifts.

/// Note that this is a problematic method if the input array is entirely 0'sE,
/// since the shifts will only add more 0's and the XORs will never produce a
/// positive bit, meaning that [0; 8] -> [0; 8], and further twiddling will only
/// produce more 0s.


fn twiddle(arr: &mut [u8; BLOCK_SIZE]) {

    for j in 0..BLOCK_SIZE {
        arr[j] ^=
            (arr[(j + 1) % BLOCK_SIZE]) << ((j + 7) % BLOCK_SIZE)
            ^ (arr[(j + 2) % BLOCK_SIZE]) << ((j + 6) % BLOCK_SIZE)
            ^ (arr[(j + 3) % BLOCK_SIZE]) << ((j + 5) % BLOCK_SIZE)
            ^ (arr[(j + 4) % BLOCK_SIZE]) << ((j + 4) % BLOCK_SIZE)
            ^ (arr[(j + 5) % BLOCK_SIZE]) >> ((j + 3) % BLOCK_SIZE)
            ^ (arr[(j + 6) % BLOCK_SIZE]) >> ((j + 2) % BLOCK_SIZE)
            ^ (arr[(j + 7) % BLOCK_SIZE]) >> ((j + 1) % BLOCK_SIZE);
    }
}

/// The transform method accepts a compress value and an array of eight bytes.
/// It XORs the array with the compress value (expressed as little-endian bytes)
/// and then runs the twiddle function on it 1,024 times.
/// The byte array is finally interpreted as a little-endian u64 and returned.

fn transform(cv: u64, arr: [u8; BLOCK_SIZE]) -> u64 {
    let mut to_return: [u8; BLOCK_SIZE] = [0; BLOCK_SIZE];
    let cv_arr: [u8; BLOCK_SIZE] = cv.to_le_bytes();

    // XOR the bytes in initial array against the CV's bytes
    for j in 0..BLOCK_SIZE {
        to_return[j] = arr[j] ^ cv_arr[j];
    }

    // For these new bytes, run the twiddle function on them 1,024 times
    for _j in 0..1024 {
        twiddle(&mut to_return);
    }

    // Return the twiddled bytes as a single u64 value by interpreting the bytes
    // as a little-endian bytes
    u64::from_le_bytes(to_return)

}

/// The compress function accepts a previous compress value and the data to operate
/// on.  It will then be converted from the `Vec<u8>` to an array (since we now know
/// that the size of the data is BLOCK_SIZE).  After that, it is transformed to give us a
/// compress value, which is then returned.
/// On the first block, the cv will equal the INITIALIZATION_VALUE.

fn compress(cv: u64, data: Vec<u8>) -> u64 {
    let mut new_data = data;
    let mut a: [u8; BLOCK_SIZE] = [0; BLOCK_SIZE];
    for j in 0..BLOCK_SIZE {
        a[j] = new_data.pop().unwrap();
    }

    transform(cv, a)
}

/// Given a vector of u8s, split it into a vector of vectors of u8s.
/// The sub-vectors of the return value should all be of size BLOCK_SIZE.
/// If the last sub-vector has less than eight elements, it should be padded
/// with 0's until it does contain eight elements.
///

fn split(data: Vec<u8>) -> Vec<Vec<u8>> {
    let to_split = strengthen(data);
    let mut to_return = Vec::new();
    let num_blocks = to_split.len() / BLOCK_SIZE;
    let mut counter = 0;
    for _j in 0..num_blocks {
        let mut new_block = Vec::new();
        for _k in 0..BLOCK_SIZE {
            new_block.push(to_split[counter]);
            counter += 1;
        }
        to_return.push(new_block);
    }
    to_return

}

/// The finalize function will return the bitwise complement of the passed-in value.
/// That is, all 0 bits will become 1 and vice versa.

fn finalize(to_finalize: u64) -> u64 {
    to_finalize ^ 0xFFFF_FFFF_FFFF_FFFF
}

/// Run the BillHash function on the input string and return the hash value.

fn bill_hash(to_hash: String) -> u64 {

    let blocks = split(convert_string_to_u8s(to_hash));
    let mut cv: u64 = INITIALIZATION_VECTOR;

    for block in blocks {
        cv = compress(cv, block);
    }

    finalize(cv)
}

/// Main function.
/// Reads a hash as the first argument from the command line and prints its
/// BillHash value.

fn main() {

    let hash_val = bill_hash(get_to_hash());
    println!("Hash value: {:#016x}", hash_val);

}

// Unit tests begin here

#[cfg(test)]
mod tests {
    use super::*;

    // ****************************************************************
    // strengthen function
    // ****************************************************************

    #[test]
    fn test_strengthen_empty_arr() {
        let to_test = Vec::new();
        let expected = [0, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(strengthen(to_test), expected);
    }

    #[test]
    fn test_strengthen_one_elem_arr() {
        let to_test = vec![1];
        let expected = [1, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(strengthen(to_test), expected);
    }

    #[test]
    fn test_strengthen_four_elem_arr() {
        let to_test = vec![0, 1, 2, 3];
        let expected = [0, 1, 2, 3, 0, 0, 0, 0];
        assert_eq!(strengthen(to_test), expected);
    }

    #[test]
    fn test_strengthen_eight_elem_arr() {
        let to_test = vec![0, 1, 2, 3, 4, 5, 6, 7];
        let expected = [0, 1, 2, 3, 4, 5, 6, 7];
        assert_eq!(strengthen(to_test), expected);
    }

    // ****************************************************************
    // split() function
    // ****************************************************************


    #[test]
    fn test_split_empty_arr() {
        let to_test = Vec::new();
        let expected = [[0, 0, 0, 0, 0, 0, 0, 0]];
        assert_eq!(split(to_test), expected);
    }

    #[test]
    fn test_split_single_elem() {
        let to_test = vec![1];
        let expected = [[1, 0, 0, 0, 0, 0, 0, 0]];
        assert_eq!(split(to_test), expected);
    }

    #[test]
    fn test_split_same_as_block_size() {
        let to_test = vec![0, 1, 2, 3, 4, 5, 6, 7];
        let expected = [[0, 1, 2, 3, 4, 5, 6, 7]];
        assert_eq!(split(to_test), expected);
    }

    #[test]
    fn test_split_one_more_than_block_size() {
        let to_test = vec![0, 1, 2, 3, 4, 5, 6, 7, 8];
        let expected = [[0, 1, 2, 3, 4, 5, 6, 7],
                        [8, 0, 0, 0, 0, 0, 0, 0]];
        assert_eq!(split(to_test), expected);
    }

    #[test]
    fn test_split_several_blocks_no_padding() {
        let to_test = vec![0, 1, 2, 3, 4, 5, 6, 7, 8,
        9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3];
        let expected = [[0, 1, 2, 3, 4, 5, 6, 7],
                        [8, 9, 0, 1, 2, 3, 4, 5],
                        [6, 7, 8, 9, 0, 1, 2, 3]];
        assert_eq!(split(to_test), expected);
    }


    #[test]
    fn test_split_several_blocks_with_padding() {
        let to_test = vec![0, 1, 2, 3, 4, 5, 6, 7, 8,
        9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let expected = [[0, 1, 2, 3, 4, 5, 6, 7],
                        [8, 9, 0, 1, 2, 3, 4, 5],
                        [6, 7, 8, 9, 0, 0, 0, 0]];
        assert_eq!(split(to_test), expected);
    }

    // ****************************************************************
    // twiddle() function
    // ****************************************************************

    #[test]
    fn test_twiddle_all_0s() {
        let mut to_test = [0; 8];
        twiddle(&mut to_test);
        assert_eq!(to_test, [0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_twiddle_all_1s() {
        let mut to_test = [1; 8];
        twiddle(&mut to_test);
        assert_eq!(to_test, [241, 220, 214, 142, 248, 177, 55, 128]);
    }

    #[test]
    fn test_twiddle_all_ffs() {
        let mut to_test = [0xFF; 8];
        twiddle(&mut to_test);
        assert_eq!(to_test, [240, 140, 167, 143, 242, 138, 87, 89]);
    }

    #[test]
    fn test_twiddle_incr() {
        let mut to_test = [0, 1, 2, 3, 4, 5, 6, 7];
        twiddle(&mut to_test);
        assert_eq!(to_test, [34, 43, 7, 158, 28, 133, 208, 242]);
    }

    #[test]
    fn test_twiddle_mix() {
        let mut to_test = [1, 2, 3, 0, 0, 0xFF, 0xAA, 0xCC];
        twiddle(&mut to_test);
        assert_eq!(to_test, [146, 214, 22, 81, 89, 204, 146, 134]);
    }

    // ****************************************************************
    // transform() function
    // ****************************************************************

    #[test]
    fn test_transform_iv_0() {
        let cv = INITIALIZATION_VECTOR;
        let to_test = [0; 8];
        assert_eq!(transform(cv, to_test), 0x2C71C76D48A512E5);
    }

    #[test]
    fn test_transform_iv_incr() {
        let cv = INITIALIZATION_VECTOR;
        let to_test = [0, 1, 2, 3, 4, 5, 6, 7];
        assert_eq!(transform(cv, to_test), 0xDF73E8863D5E2E4);

    }

    #[test]
    fn test_transform_iv_all_1s() {
        let cv = INITIALIZATION_VECTOR;
        let to_test = [1; 8];
        assert_eq!(transform(cv, to_test), 0xC44441A4484800A3);

    }

    #[test]
    fn test_transform_iv_all_ffs() {
        let cv = INITIALIZATION_VECTOR;
        let to_test = [0xFF; 8];
        assert_eq!(transform(cv, to_test), 0xDB47BA4E73CAF7F5);

    }

    // ****************************************************************
    // compress() function
    // ****************************************************************

    #[test]
    fn test_compress_0_0() {
        let cv = 0;
        let to_test = vec![0; 8];
        assert_eq!(compress(cv, to_test), 0x0);
    }

    #[test]
    fn test_compress_0_ffs() {
        let cv = 0;
        let to_test = vec![0xFF; 8];
        assert_eq!(compress(cv, to_test), 0xF7367D233B6FE510);
    }

    #[test]
    fn test_compress_iv_0() {
        let cv = INITIALIZATION_VECTOR;
        let to_test = vec![0; 8];
        assert_eq!(compress(cv, to_test), 0x2C71C76D48A512E5);
    }

    #[test]
    fn test_compress_iv_incr() {
        let cv = INITIALIZATION_VECTOR;
        let to_test = vec![0, 1, 2, 3, 4, 5, 6, 7];
        assert_eq!(compress(cv, to_test), 0x43FC4E68B1A699B8);
    }

    // ****************************************************************
    // finalize() function
    // ****************************************************************

    #[test]
    fn test_finalize_0() {
        assert_eq!(finalize(0), 0xFFFFFFFFFFFFFFFF);
    }

    #[test]
    fn test_finalize_maxint() {
        assert_eq!(finalize(0xFFFFFFFFFFFFFFFF), 0);
    }

    #[test]
    fn test_finalize_mixed() {
        assert_eq!(finalize(0xF0F0F0F0F0F0F0F0), 0x0F0F0F0F0F0F0F0F);
    }

    #[test]
    fn test_finalize_some_value() {
        assert_eq!(finalize(3987120094), 18446744069722431521);
    }

    #[test]
    fn test_finalize_some_value_2() {
        assert_eq!(finalize(0x123456789ABCDEF), 0xFEDCBA9876543210);
    }

    #[test]
    fn test_finalize_some_value_3() {
        assert_eq!(finalize(0xDEADBEEFDEADBEEF), 0x2152411021524110);
    }


    // ****************************************************************
    // bill_hash() function
    // ****************************************************************

    #[test]
    fn test_hash_empty() {
        assert_eq!(bill_hash("".to_string()), 0xd38e3892b75aed1a);
    }

    #[test]
    fn test_hash_very_small() {
        assert_eq!(bill_hash("b".to_string()), 0x7DACF192C75DB1DB);

    }

    #[test]
    fn test_hash_bill() {
        assert_eq!(bill_hash("bill".to_string()), 0x45AAEC6CD9F47E66);

    }

    #[test]
    fn test_hash_hash() {
        assert_eq!(bill_hash("hash".to_string()), 0xFE75BD197EA432C9);

    }

    #[test]
    fn test_hash_long_entry() {
        let long_string = "It was the best of times, it was the worst of times, it was the age of wisdom, it was the age of foolishness, it was the epoch of belief, it was the epoch of incredulity, it was the season of Light, it was the season of Darkness, it was the spring of hope, it was the winter of despair, we had everything before us, we had nothing before us, we were all going direct to heaven, we were all going direct the other way - in short, the period was so far like the present period, that some of its noisiest authorities insisted on its being received, for good or for evil, in the superlative degree of comparison only.".to_string();
        assert_eq!(bill_hash(long_string), 0x7391BAE2DB358FF4);

    }




}
