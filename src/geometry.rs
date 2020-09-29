use cgmath::*;

#[derive(Debug, Copy, Clone)]
pub struct ColoredVertex
{
	#[allow(dead_code)]
	position : Vector3<f32>,

	#[allow(dead_code)]
	color : Vector4<f32>
}

#[allow(dead_code)]
const SAMPLE_ASPECT_RATIO : f32 = 1280.0/720.0;

// This sample triangle is already in projection space.
#[allow(dead_code)]
const SAMPLE_PROJECTED_TRIANGLE_VERTICES : [Vector3<f32>; 3] =
[
	Vector3::new(0.0, 0.25 * SAMPLE_ASPECT_RATIO, 0.0),
	Vector3::new(0.25, -0.25 * SAMPLE_ASPECT_RATIO, 0.0),
	Vector3::new(-0.25, -0.25 * SAMPLE_ASPECT_RATIO, 0.0),
];

#[allow(dead_code)]
pub fn sample_colored_triangle_vertices(aspect_ratio : f32) -> [ColoredVertex; 3]
{
	return 
	[
		ColoredVertex 
		{ 
			position : Vector3::new(0.0, 0.25 * aspect_ratio, 0.0), 
			color : Vector4::new(1.0, 0.0, 0.0, 1.0)
		},
		ColoredVertex 
		{ 
			position : Vector3::new(0.25, -0.25 * aspect_ratio, 0.0), 
			color : Vector4::new(0.0, 1.0, 0.0, 1.0)
		},
		ColoredVertex 
		{ 
			position : Vector3::new(-0.25, -0.25 * aspect_ratio, 0.0), 
			color : Vector4::new(0.0, 0.0, 1.0, 1.0)
		},
	]
}

// Sample colored tetrahedron in Projection space.
#[allow(dead_code)]
pub fn sample_colored_tetrahedron_vertices() -> [ColoredVertex; 12]
{
	// let eye = Point3::new(0.0, 0.66, -2.0);
	// let target = Point3::new(0.0, 0.66, 0.0);
	// let up = Vector3::unit_y();
	
	// let view_lh = transforms::look_at_lh(eye, target, up);

	// let perspective = PerspectiveFov {
	// 	fovy : cgmath::Rad(fovy.to_radians()), 
	// 	aspect : aspect_ratio,
	// 	near : 0.1,
	// 	far : 100.0};

	// let proj_lh = transforms::perspective_lh(perspective);
	
	// let view_proj = proj_lh * view_lh;

	// Vertices for a unit tetrahedron in world space, with center of the bottom face at the origin.
	let sample_tetrahedron_vertices : [Vector3<f32>; 4] =
	[
		Vector3::new(-1.0, 0.0, -1.0/3.0_f32.sqrt()),
		Vector3::new(1.0 , 0.0, -1.0/3.0_f32.sqrt()),
		Vector3::new(0.0 , 0.0, 2.0/3.0_f32.sqrt()),
		Vector3::new(0.0, 4.0/6.0_f32.sqrt(), 0.0)
	];

	// // trasform to projection space
	// let vleft = view_proj * sample_tetrahedron_vertices[0].extend(1.0);
	// let vright = view_proj * sample_tetrahedron_vertices[1].extend(1.0);
	// let vback = view_proj * sample_tetrahedron_vertices[2].extend(1.0);
	// let vtop = view_proj * sample_tetrahedron_vertices[3].extend(1.0);

	// // perspective divide (normally done by gpu pipeline)
	// let vvleft = vleft.map(|c| c / vleft.w).truncate();
	// let vvright = vright.map(|c| c / vright.w).truncate();
	// let vvback = vback.map(|c| c / vback.w).truncate();
	// let vvtop = vtop.map(|c| c / vtop.w).truncate();

	let left = sample_tetrahedron_vertices[0];
	let right = sample_tetrahedron_vertices[1];
	let back = sample_tetrahedron_vertices[2];
	let top = sample_tetrahedron_vertices[3];
	
	let blue = Vector4::new(0.0, 0.0, 1.0, 1.0);
	let green = Vector4::new(0.0, 1.0, 0.0, 1.0);
	let red = Vector4::new(1.0, 0.0, 0.0, 1.0);
	let yellow =  Vector4::new(1.0, 1.0, 0.0, 1.0);

	let left = ColoredVertex { position : left, color : blue };
	let top = ColoredVertex { position : top, color : green };
	let right = ColoredVertex { position : right, color : red };
	let back = ColoredVertex { position : back, color : yellow };
	

	let colored_vertices = 
	[
		left,
		top, 
		right,

		back,
		top,
		left,
		
		right,
		top,
		back,

		left,
		right,
		back,
	];

	return colored_vertices;
}