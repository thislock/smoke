
struct VertexInput {
  @location(0) position: vec3<f32>,
  @location(1) color: vec3<f32>,
};

struct VertexOutput {
  @builtin(position) clip_position: vec4<f32>,
  @location(0) color: vec3f,
};

@vertex
fn vs_main(
  model: VertexInput,
) -> VertexOutput {
  var out: VertexOutput;
  out.color = model.color;
  out.clip_position = vec4f(model.position, 1.0);
  return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  // convert the sRGB color into RGB (what wpu expects, relative to what the user expects)
  let a = ((in.color + 0.055) / 1.055);
  let sRGB = pow(a, vec3f(2.4));
  return vec4<f32>(sRGB, 1.0);
}
