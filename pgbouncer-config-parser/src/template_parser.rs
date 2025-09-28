pub trait TemplateParser: Sized {
    type Error: std::error::Error;
    
    fn to_template_string(&self) -> Result<String, Self::Error>;
    fn from_template_string(s: &str) -> Result<Self, Self::Error>;
}