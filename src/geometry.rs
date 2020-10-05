use cgmath::*;
use crate::transforms;

#[derive(Debug, Copy, Clone)]
pub struct ColoredVertex
{
	#[allow(dead_code)]
	position : Vector3<f32>,

	#[allow(dead_code)]
	color : Vector4<f32>,
}

#[allow(dead_code)]
const SAMPLE_ASPECT_RATIO : f32 = 1280.0 / 720.0;

// This sample triangle is already in projection space.
#[allow(dead_code)]
const SAMPLE_PROJECTED_TRIANGLE_VERTICES : [Vector3<f32>; 3] = [
	Vector3::new(0.0, 0.25 * SAMPLE_ASPECT_RATIO, 0.0),
	Vector3::new(0.25, -0.25 * SAMPLE_ASPECT_RATIO, 0.0),
	Vector3::new(-0.25, -0.25 * SAMPLE_ASPECT_RATIO, 0.0),
];

const BLUE : Vector4<f32> = Vector4::new(0.0, 0.0, 1.0, 1.0);
const GREEN : Vector4<f32> = Vector4::new(0.0, 1.0, 0.0, 1.0);
const RED : Vector4<f32> = Vector4::new(1.0, 0.0, 0.0, 1.0);
const YELLOW : Vector4<f32> = Vector4::new(1.0, 1.0, 0.0, 1.0);

#[allow(dead_code)]
pub fn sample_colored_triangle_vertices_pre_projected(aspect_ratio : f32) -> [ColoredVertex; 3]
{
	return [
		// left
		ColoredVertex {
			position : Vector3::new(-0.25, -0.25 * aspect_ratio, 0.0),
			color :    BLUE,
		},
		// top
		ColoredVertex {
			position : Vector3::new(0.0, 0.25 * aspect_ratio, 0.0),
			color :    GREEN,
		},
		// right
		ColoredVertex {
			position : Vector3::new(0.25, -0.25 * aspect_ratio, 0.0),
			color :    RED,
		},
	];
}

#[allow(dead_code)]
pub fn sample_colored_triangle_vertices_eye_pre_projected(fovy : f32, aspect_ratio : f32) -> [ColoredVertex; 3]
{
	// Create View transform
	let eye = Point3::new(0.0, 0.66, -2.0);
	let target = Point3::new(0.0, 0.66, 0.0);
	let up = Vector3::unit_y();

	let view_lh = transforms::look_at_lh(eye, target, up);

	// Create Perspective Projection transform
	let perspective = PerspectiveFov {
		fovy : cgmath::Rad(fovy.to_radians()),
		aspect : aspect_ratio,
		near : 0.1,
		far : 100.0};

	// Create the view-Projection transform
	let proj_lh = transforms::perspective_lh(perspective);
	let view_proj = proj_lh * view_lh;

	// Sample Vertices for an Equilateral triangle (might be a little off from truly equilateral)
	let sample_triangle_vertices : [Vector3<f32>; 3] = [
		Vector3::new(-1.0, 0.0, 0.0),
		Vector3::new(0.0, 4.0 / 6.0_f32.sqrt(), 0.0),
		Vector3::new(1.0, 0.0, 0.0),
	];

	// trasform to projection space
	let vleft  = view_proj * sample_triangle_vertices[0].extend(1.0);
	let vtop   = view_proj * sample_triangle_vertices[1].extend(1.0);
	let vright = view_proj * sample_triangle_vertices[2].extend(1.0);

	// perspective divide (normally done by gpu pipeline)
	let vvleft = vleft.map(|c| c / vleft.w).truncate();
	let vvtop = vtop.map(|c| c / vtop.w).truncate();
	let vvright = vright.map(|c| c / vright.w).truncate();

	// Assemble Colored Vertex Array
	let left = ColoredVertex {
		position : vvleft,
		color :    BLUE,
	};
	let top = ColoredVertex {
		position : vvtop,
		color :    GREEN,
	};
	let right = ColoredVertex {
		position : vvright,
		color :    RED,
	};

	let colored_vertices = [left, top, right];

	return colored_vertices;
}

// Sample colored tetrahedron in Projection space.
#[allow(dead_code)]
pub fn sample_colored_tetrahedron_vertices() -> [ColoredVertex; 12]
{
	// Vertices for a unit tetrahedron in world space, with center of the bottom
	// face at the origin.
	let sample_tetrahedron_vertices : [Vector3<f32>; 4] = [
		Vector3::new(-1.0, 0.0, -1.0 / 3.0_f32.sqrt()),
		Vector3::new(1.0, 0.0, -1.0 / 3.0_f32.sqrt()),
		Vector3::new(0.0, 0.0, 2.0 / 3.0_f32.sqrt()),
		Vector3::new(0.0, 4.0 / 6.0_f32.sqrt(), 0.0),
	];

	let left = sample_tetrahedron_vertices[0];
	let right = sample_tetrahedron_vertices[1];
	let back = sample_tetrahedron_vertices[2];
	let top = sample_tetrahedron_vertices[3];

	let left = ColoredVertex {
		position : left,
		color :    BLUE,
	};
	let top = ColoredVertex {
		position : top,
		color :    GREEN,
	};
	let right = ColoredVertex {
		position : right,
		color :    RED,
	};
	let back = ColoredVertex {
		position : back,
		color :    YELLOW,
	};

	let colored_vertices = [left, top, right, back, top, left, right, top, back, left, right, back];

	return colored_vertices;
}
