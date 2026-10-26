//! Sistema de diseño: los valores con nombre que comparten todas las apps de la suite. Un color o
//! una medida nueva se agrega aquí, no suelta en una vista.
//!
//! Estilo: neutro y tranquilo (grises fríos, un solo acento azul, el marino de la marca para la
//! identidad), sin cintas ni degradados. Contraste AA (4.5:1) para el texto normal y el secundario
//! sobre todos los fondos; lo verifican las pruebas de [`crate::theme`].

use egui::Color32;

/// Colores de la marca (familia de apps Iure: el icono y la identidad).
pub mod brand {
    use egui::Color32;

    /// Marino Iurefficient (fondo del icono de las apps).
    pub const NAVY: Color32 = Color32::from_rgb(17, 40, 76);
    /// Marino aclarado para el tema oscuro (sobre fondo casi negro).
    pub const NAVY_DARK_THEME: Color32 = Color32::from_rgb(30, 58, 104);
    /// Azul del medio círculo del logotipo.
    pub const BLUE: Color32 = Color32::from_rgb(6, 80, 236);
    /// Verde de la barra del logotipo.
    pub const GREEN: Color32 = Color32::from_rgb(34, 197, 94);
}

/// Colores con función de un tema. Todos opacos salvo `hover` y `pressed`, que se mezclan sobre
/// el fondo que tengan debajo.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Palette {
    /// ¿Es un tema oscuro?
    pub dark: bool,
    /// Fondo de la ventana (detrás de los paneles).
    pub bg: Color32,
    /// Superficie principal: listas, lectura, campos, ventanas flotantes.
    pub surface: Color32,
    /// Superficie secundaria: barra lateral, riel, encabezados de tabla.
    pub surface_alt: Color32,
    /// Texto normal.
    pub text: Color32,
    /// Texto secundario (fechas, sugerencias, vista previa).
    pub text_muted: Color32,
    /// Acento: botón principal, selección activa, enlaces, insignias.
    pub accent: Color32,
    /// Texto sobre el acento.
    pub on_accent: Color32,
    /// Peligro: borrar, errores.
    pub danger: Color32,
    /// Atención: avisos, banderas de seguimiento.
    pub warning: Color32,
    /// Hecho: confirmaciones, tareas terminadas.
    pub success: Color32,
    /// Información (avisos neutros).
    pub info: Color32,
    /// Fondo de las filas seleccionadas.
    pub selection: Color32,
    /// Fondo de la selección cuando la lista no tiene el foco.
    pub selection_inactive: Color32,
    /// Anillo de foco del teclado.
    pub focus: Color32,
    /// Separadores y bordes de los controles.
    pub separator: Color32,
    /// Capa translúcida al pasar el ratón.
    pub hover: Color32,
    /// Capa translúcida al presionar.
    pub pressed: Color32,
    /// Fondo de las filas marcadas con bandera.
    pub flag_tint: Color32,
    /// Marino de la marca en este tema.
    pub brand: Color32,
}

/// Tema claro.
pub const LIGHT: Palette = Palette {
    dark: false,
    bg: Color32::from_rgb(243, 244, 246),
    surface: Color32::from_rgb(255, 255, 255),
    surface_alt: Color32::from_rgb(248, 249, 250),
    text: Color32::from_rgb(28, 30, 34),
    text_muted: Color32::from_rgb(92, 98, 110),
    accent: Color32::from_rgb(29, 88, 208),
    on_accent: Color32::from_rgb(255, 255, 255),
    danger: Color32::from_rgb(190, 40, 34),
    warning: Color32::from_rgb(166, 90, 0),
    success: Color32::from_rgb(22, 122, 66),
    info: Color32::from_rgb(29, 88, 208),
    selection: Color32::from_rgb(214, 228, 252),
    selection_inactive: Color32::from_rgb(230, 233, 238),
    focus: Color32::from_rgb(29, 88, 208),
    separator: Color32::from_rgb(222, 225, 230),
    hover: Color32::from_rgba_premultiplied(0, 0, 0, 12),
    pressed: Color32::from_rgba_premultiplied(0, 0, 0, 24),
    flag_tint: Color32::from_rgb(255, 246, 228),
    brand: brand::NAVY,
};

/// Tema oscuro.
pub const DARK: Palette = Palette {
    dark: true,
    bg: Color32::from_rgb(22, 23, 26),
    surface: Color32::from_rgb(30, 31, 35),
    surface_alt: Color32::from_rgb(26, 27, 31),
    text: Color32::from_rgb(230, 232, 235),
    text_muted: Color32::from_rgb(168, 174, 184),
    accent: Color32::from_rgb(110, 160, 250),
    on_accent: Color32::from_rgb(12, 20, 36),
    danger: Color32::from_rgb(255, 118, 108),
    warning: Color32::from_rgb(240, 176, 72),
    success: Color32::from_rgb(92, 204, 132),
    info: Color32::from_rgb(110, 160, 250),
    selection: Color32::from_rgb(38, 60, 102),
    selection_inactive: Color32::from_rgb(48, 51, 58),
    focus: Color32::from_rgb(130, 175, 255),
    separator: Color32::from_rgb(52, 54, 60),
    hover: Color32::from_rgba_premultiplied(255, 255, 255, 12),
    pressed: Color32::from_rgba_premultiplied(255, 255, 255, 24),
    flag_tint: Color32::from_rgb(52, 44, 30),
    brand: brand::NAVY_DARK_THEME,
};

/// Alto contraste: fondo negro, texto blanco, bordes y selección muy marcados.
pub const HIGH_CONTRAST: Palette = Palette {
    dark: true,
    bg: Color32::from_rgb(0, 0, 0),
    surface: Color32::from_rgb(0, 0, 0),
    surface_alt: Color32::from_rgb(16, 16, 16),
    text: Color32::from_rgb(255, 255, 255),
    text_muted: Color32::from_rgb(220, 220, 220),
    accent: Color32::from_rgb(120, 180, 255),
    on_accent: Color32::from_rgb(0, 0, 0),
    danger: Color32::from_rgb(255, 120, 120),
    warning: Color32::from_rgb(255, 200, 60),
    success: Color32::from_rgb(110, 230, 140),
    info: Color32::from_rgb(120, 180, 255),
    selection: Color32::from_rgb(0, 70, 170),
    selection_inactive: Color32::from_rgb(60, 60, 60),
    focus: Color32::from_rgb(255, 255, 0),
    separator: Color32::from_rgb(200, 200, 200),
    hover: Color32::from_rgba_premultiplied(255, 255, 255, 40),
    pressed: Color32::from_rgba_premultiplied(255, 255, 255, 70),
    flag_tint: Color32::from_rgb(70, 50, 0),
    brand: brand::NAVY_DARK_THEME,
};

/// Espaciado, en puntos lógicos (múltiplos de 4, con 2 para ajustes finos).
pub mod space {
    pub const XS: f32 = 2.0;
    pub const S: f32 = 4.0;
    pub const M: f32 = 8.0;
    pub const L: f32 = 12.0;
    pub const XL: f32 = 16.0;
    pub const XXL: f32 = 24.0;
    pub const XXXL: f32 = 32.0;
}

/// Radios de las esquinas.
pub mod radius {
    /// Marcas pequeñas (barras, insignias cuadradas).
    pub const SMALL: f32 = 2.0;
    /// Botones, campos y filas resaltadas.
    pub const CONTROL: f32 = 6.0;
    /// Tarjetas, avisos y menús.
    pub const CARD: f32 = 8.0;
    /// Ventanas y cuadros flotantes.
    pub const WINDOW: f32 = 10.0;
    /// Píldora (insignias, controles segmentados).
    pub const PILL: f32 = 999.0;
}

/// Tamaños de letra base (densidad cómoda), en puntos lógicos. La densidad los ajusta.
///
/// Escala pensada para Inter (altura de x de 0.55 em: a igual tamaño se lee más grande que Noto
/// o Source Sans): cuerpo de 14/13/12.5 según la densidad, como el 13 de macOS; títulos en
/// seminegrita en lugar de agrandarlos. egui no tiene espaciado entre letras, así que los
/// tamaños grandes bajan un poco en vez de apretar el tracking.
pub mod text {
    pub const SMALL: f32 = 11.5;
    pub const BODY: f32 = 14.0;
    pub const HEADING: f32 = 18.0;
    pub const TITLE: f32 = 24.0;
    /// Título de la vista de lectura (asunto de un mensaje, nombre de un contacto).
    pub const SUBTITLE: f32 = 20.0;
}

/// Densidad de la interfaz: alto de las filas, márgenes y tamaño de letra. «Densa» supera a
/// Outlook (filas de 22 px, una línea).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Density {
    #[default]
    Comfortable,
    Compact,
    Dense,
}

impl Density {
    pub const ALL: [Density; 3] = [Density::Comfortable, Density::Compact, Density::Dense];

    /// Alto de una fila de lista o tabla de una línea.
    pub fn row_height(self) -> f32 {
        match self {
            Density::Comfortable => 34.0,
            Density::Compact => 28.0,
            Density::Dense => 22.0,
        }
    }

    /// Tamaño del texto normal.
    pub fn body_size(self) -> f32 {
        match self {
            Density::Comfortable => text::BODY,
            Density::Compact => 13.0,
            Density::Dense => 12.5,
        }
    }

    /// Tamaño de los títulos de sección (`TextStyle::Heading`, en seminegrita).
    pub fn heading_size(self) -> f32 {
        match self {
            Density::Comfortable => text::HEADING,
            Density::Compact => 16.0,
            Density::Dense => 15.0,
        }
    }

    /// Tamaño del texto pequeño (fechas, notas).
    pub fn small_size(self) -> f32 {
        match self {
            Density::Comfortable => 12.0,
            Density::Compact => text::SMALL,
            Density::Dense => 11.0,
        }
    }

    /// Lado de los iconos en botones y filas.
    pub fn icon_size(self) -> f32 {
        match self {
            Density::Comfortable => 18.0,
            Density::Compact => 16.0,
            Density::Dense => 15.0,
        }
    }

    /// Margen interior de los botones (horizontal, vertical).
    pub fn button_padding(self) -> egui::Vec2 {
        match self {
            Density::Comfortable => egui::vec2(10.0, 5.0),
            Density::Compact => egui::vec2(8.0, 3.0),
            Density::Dense => egui::vec2(6.0, 2.0),
        }
    }

    /// Separación entre controles (horizontal, vertical).
    pub fn item_spacing(self) -> egui::Vec2 {
        match self {
            Density::Comfortable => egui::vec2(8.0, 6.0),
            Density::Compact => egui::vec2(6.0, 4.0),
            Density::Dense => egui::vec2(6.0, 3.0),
        }
    }

    /// Alto mínimo de un control interactivo (botón, campo).
    pub fn control_height(self) -> f32 {
        match self {
            Density::Comfortable => 30.0,
            Density::Compact => 26.0,
            Density::Dense => 22.0,
        }
    }

    /// Margen horizontal de las celdas de una tabla.
    pub fn cell_padding(self) -> f32 {
        match self {
            Density::Comfortable => 10.0,
            Density::Compact => 8.0,
            Density::Dense => 6.0,
        }
    }

    pub fn label_en(self) -> &'static str {
        match self {
            Density::Comfortable => "Comfortable",
            Density::Compact => "Compact",
            Density::Dense => "Dense",
        }
    }

    pub fn label_es(self) -> &'static str {
        match self {
            Density::Comfortable => "Cómoda",
            Density::Compact => "Compacta",
            Density::Dense => "Densa",
        }
    }

    /// Nombre en el idioma de la interfaz.
    pub fn label(self) -> &'static str {
        crate::lang::pick(self.label_en(), self.label_es())
    }

    /// Nombre corto estable para guardar en preferencias.
    pub fn key(self) -> &'static str {
        match self {
            Density::Comfortable => "comfortable",
            Density::Compact => "compact",
            Density::Dense => "dense",
        }
    }

    /// Lee un nombre guardado con [`Density::key`] (acepta también los nombres en español).
    pub fn from_key(s: &str) -> Option<Density> {
        match s.trim().to_ascii_lowercase().as_str() {
            "comfortable" | "comoda" | "cómoda" => Some(Density::Comfortable),
            "compact" | "compacta" => Some(Density::Compact),
            "dense" | "densa" => Some(Density::Dense),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dense_rows_are_22px_and_heights_grow_with_density() {
        assert_eq!(Density::Dense.row_height(), 22.0);
        assert!(Density::Compact.row_height() > Density::Dense.row_height());
        assert!(Density::Comfortable.row_height() > Density::Compact.row_height());
        for d in Density::ALL {
            assert_eq!(Density::from_key(d.key()), Some(d));
            // El texto cabe en la fila con aire.
            assert!(d.body_size() * 1.3 < d.row_height());
        }
    }
}
