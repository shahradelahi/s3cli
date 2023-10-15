pub struct DuOutput {
  pub total_size_bytes: usize,
  pub total_size_human: String,
  pub total_objects: usize,
  pub objects: Vec<aws_sdk_s3::types::Object>,
}