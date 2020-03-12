// This file was generated by gir (https://github.com/gtk-rs/gir)
// from gir-files (https://github.com/gtk-rs/gir-files)
// DO NOT EDIT

use aravis_sys;
use glib;
use glib::object::IsA;
use glib::translate::*;
use glib::GString;
use std::fmt;
use std::ptr;
use DomElement;
use DomNode;
use GcNode;

glib_wrapper! {
	pub struct GcFeatureNode(Object<aravis_sys::ArvGcFeatureNode, aravis_sys::ArvGcFeatureNodeClass, GcFeatureNodeClass>) @extends GcNode, DomElement, DomNode;

	match fn {
		get_type => || aravis_sys::arv_gc_feature_node_get_type(),
	}
}

pub const NONE_GC_FEATURE_NODE: Option<&GcFeatureNode> = None;

pub trait GcFeatureNodeExt: 'static {
	fn get_description(&self) -> Option<GString>;

	fn get_display_name(&self) -> Option<GString>;

	fn get_name(&self) -> Option<GString>;

	fn get_tooltip(&self) -> Option<GString>;

	fn get_value_as_string(&self) -> Result<GString, glib::Error>;

	//fn get_visibility(&self) -> /*Ignored*/GcVisibility;

	fn is_available(&self) -> Result<bool, glib::Error>;

	fn is_implemented(&self) -> Result<bool, glib::Error>;

	fn is_locked(&self) -> Result<bool, glib::Error>;

	fn set_value_from_string(&self, string: &str) -> Result<(), glib::Error>;
}

impl<O: IsA<GcFeatureNode>> GcFeatureNodeExt for O {
	fn get_description(&self) -> Option<GString> {
		unsafe {
			from_glib_none(aravis_sys::arv_gc_feature_node_get_description(
				self.as_ref().to_glib_none().0,
			))
		}
	}

	fn get_display_name(&self) -> Option<GString> {
		unsafe {
			from_glib_none(aravis_sys::arv_gc_feature_node_get_display_name(
				self.as_ref().to_glib_none().0,
			))
		}
	}

	fn get_name(&self) -> Option<GString> {
		unsafe {
			from_glib_none(aravis_sys::arv_gc_feature_node_get_name(
				self.as_ref().to_glib_none().0,
			))
		}
	}

	fn get_tooltip(&self) -> Option<GString> {
		unsafe {
			from_glib_none(aravis_sys::arv_gc_feature_node_get_tooltip(
				self.as_ref().to_glib_none().0,
			))
		}
	}

	fn get_value_as_string(&self) -> Result<GString, glib::Error> {
		unsafe {
			let mut error = ptr::null_mut();
			let ret = aravis_sys::arv_gc_feature_node_get_value_as_string(
				self.as_ref().to_glib_none().0,
				&mut error,
			);
			if error.is_null() {
				Ok(from_glib_none(ret))
			} else {
				Err(from_glib_full(error))
			}
		}
	}

	//fn get_visibility(&self) -> /*Ignored*/GcVisibility {
	//    unsafe { TODO: call aravis_sys:arv_gc_feature_node_get_visibility() }
	//}

	fn is_available(&self) -> Result<bool, glib::Error> {
		unsafe {
			let mut error = ptr::null_mut();
			let ret = aravis_sys::arv_gc_feature_node_is_available(
				self.as_ref().to_glib_none().0,
				&mut error,
			);
			if error.is_null() {
				Ok(from_glib(ret))
			} else {
				Err(from_glib_full(error))
			}
		}
	}

	fn is_implemented(&self) -> Result<bool, glib::Error> {
		unsafe {
			let mut error = ptr::null_mut();
			let ret = aravis_sys::arv_gc_feature_node_is_implemented(
				self.as_ref().to_glib_none().0,
				&mut error,
			);
			if error.is_null() {
				Ok(from_glib(ret))
			} else {
				Err(from_glib_full(error))
			}
		}
	}

	fn is_locked(&self) -> Result<bool, glib::Error> {
		unsafe {
			let mut error = ptr::null_mut();
			let ret = aravis_sys::arv_gc_feature_node_is_locked(
				self.as_ref().to_glib_none().0,
				&mut error,
			);
			if error.is_null() {
				Ok(from_glib(ret))
			} else {
				Err(from_glib_full(error))
			}
		}
	}

	fn set_value_from_string(&self, string: &str) -> Result<(), glib::Error> {
		unsafe {
			let mut error = ptr::null_mut();
			let _ = aravis_sys::arv_gc_feature_node_set_value_from_string(
				self.as_ref().to_glib_none().0,
				string.to_glib_none().0,
				&mut error,
			);
			if error.is_null() {
				Ok(())
			} else {
				Err(from_glib_full(error))
			}
		}
	}
}

impl fmt::Display for GcFeatureNode {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "GcFeatureNode")
	}
}
