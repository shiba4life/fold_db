mod test_helpers;
use test_helpers::cleanup_tmp_dir;

// Clean up tmp directory after all tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleanup() {
        cleanup_tmp_dir();
    }
}
