pub const STEAM_ICON: egui::ImageSource<'static> =
    egui::include_image!("../../assets/icons/steam.png");
pub const EPIC_ICON: egui::ImageSource<'static> =
    egui::include_image!("../../assets/icons/epicgames.png");
pub const UBISOFT_ICON: egui::ImageSource<'static> =
    egui::include_image!("../../assets/icons/ubisoft.png");
pub const EA_ICON: egui::ImageSource<'static> = egui::include_image!("../../assets/icons/ea.png");
pub const BATTLENET_ICON: egui::ImageSource<'static> =
    egui::include_image!("../../assets/icons/battledotnet.png");
pub const RETROARCH_ICON: egui::ImageSource<'static> =
    egui::include_image!("../../assets/icons/retroarch.png");
pub const GOG_ICON: egui::ImageSource<'static> =
    egui::include_image!("../../assets/icons/gogdotcom.png");
pub const XBOX_ICON: egui::ImageSource<'static> =
    egui::include_image!("../../assets/icons/xbox.png");

#[allow(dead_code)]
pub fn get_platform_icon(platform: crate::models::Platform) -> egui::ImageSource<'static> {
    match platform {
        crate::models::Platform::Steam => STEAM_ICON,
        crate::models::Platform::EpicGames => EPIC_ICON,
        crate::models::Platform::Ubisoft => UBISOFT_ICON,
        crate::models::Platform::Ea => EA_ICON,
        crate::models::Platform::BattleNet => BATTLENET_ICON,
        crate::models::Platform::Emulator => RETROARCH_ICON,
        crate::models::Platform::Gog => GOG_ICON,
        crate::models::Platform::Xbox => XBOX_ICON,
        crate::models::Platform::Custom => STEAM_ICON,
    }
}
