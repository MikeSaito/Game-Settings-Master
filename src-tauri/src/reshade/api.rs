use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphicsApi {
    Dx9,
    Dx11,
    Dx12,
    OpenGL,
    Vulkan,
}

#[derive(Debug, Clone, Serialize)]
pub struct GraphicsApiInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub files: Vec<String>,
}

impl GraphicsApi {
    pub fn all() -> &'static [GraphicsApi] {
        &[
            GraphicsApi::Dx9,
            GraphicsApi::Dx11,
            GraphicsApi::Dx12,
            GraphicsApi::OpenGL,
            GraphicsApi::Vulkan,
        ]
    }

    pub fn from_str_id(id: &str) -> Result<Self, String> {
        match id.trim().to_ascii_lowercase().as_str() {
            "dx9" => Ok(GraphicsApi::Dx9),
            "dx11" | "dx10_11" | "d3d11" => Ok(GraphicsApi::Dx11),
            "dx12" | "dxgi" => Ok(GraphicsApi::Dx12),
            "opengl" | "ogl" => Ok(GraphicsApi::OpenGL),
            "vulkan" | "vk" => Ok(GraphicsApi::Vulkan),
            other => Err(format!("Неизвестный графический API: {other}")),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            GraphicsApi::Dx9 => "dx9",
            GraphicsApi::Dx11 => "dx11",
            GraphicsApi::Dx12 => "dx12",
            GraphicsApi::OpenGL => "opengl",
            GraphicsApi::Vulkan => "vulkan",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            GraphicsApi::Dx9 => "DirectX 9",
            GraphicsApi::Dx11 => "DirectX 10/11",
            GraphicsApi::Dx12 => "DirectX 12",
            GraphicsApi::OpenGL => "OpenGL",
            GraphicsApi::Vulkan => "Vulkan",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            GraphicsApi::Dx9 => "Старые игры. Устанавливает d3d9.dll.",
            GraphicsApi::Dx11 => {
                "Классический D3D11 — d3d11.dll. Старые UE4, многие инди. Если не работает — попробуйте DX12."
            }
            GraphicsApi::Dx12 => {
                "DX12 и DXGI-игры — dxgi.dll. Обычно UE5 и современные AAA."
            }
            GraphicsApi::OpenGL => "OpenGL-рендер — opengl32.dll.",
            GraphicsApi::Vulkan => {
                "Vulkan implicit layer: ReShade64.dll + ReShade64.json (реестр, как в официальном setup)."
            }
        }
    }

    pub fn files_to_install(self) -> &'static [&'static str] {
        match self {
            GraphicsApi::Dx9 => &["d3d9.dll"],
            GraphicsApi::Dx11 => &["d3d11.dll"],
            GraphicsApi::Dx12 => &["dxgi.dll"],
            GraphicsApi::OpenGL => &["opengl32.dll"],
            GraphicsApi::Vulkan => &["ReShade64.dll", "ReShade64.json"],
        }
    }

}

/// Known proxy/layer filenames GSM may reference from an install marker.
pub fn is_known_install_filename(name: &str) -> bool {
    GraphicsApi::all()
        .iter()
        .flat_map(|api| api.files_to_install())
        .any(|known| *known == name)
}

pub fn list_graphics_apis() -> Vec<GraphicsApiInfo> {
    GraphicsApi::all()
        .iter()
        .map(|api| GraphicsApiInfo {
            id: api.as_str().to_string(),
            name: api.display_name().to_string(),
            description: api.description().to_string(),
            files: api.files_to_install().iter().map(|f| (*f).to_string()).collect(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_api_ids() {
        assert_eq!(GraphicsApi::from_str_id("dx12").unwrap(), GraphicsApi::Dx12);
        assert_eq!(GraphicsApi::from_str_id("dx11").unwrap(), GraphicsApi::Dx11);
        assert!(GraphicsApi::from_str_id("unknown").is_err());
    }

    #[test]
    fn dx12_uses_dxgi() {
        assert_eq!(GraphicsApi::Dx12.files_to_install()[0], "dxgi.dll");
    }

    #[test]
    fn vulkan_installs_layer_files() {
        assert_eq!(
            GraphicsApi::Vulkan.files_to_install(),
            &["ReShade64.dll", "ReShade64.json"]
        );
    }
}
