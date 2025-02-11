mod test_utils;
use test_utils::cleanup_tmp_dir;

// Clean up tmp directory after all tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleanup() {
        cleanup_tmp_dir();
    }
}
