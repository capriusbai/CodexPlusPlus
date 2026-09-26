pub const VERSION: &str = "2.0.0";

#[cfg(test)]
mod tests {
    use super::VERSION;

    #[test]
    fn exposes_workspace_version() {
        assert_eq!(VERSION, "2.0.0");
    }
}
