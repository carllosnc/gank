pub mod battlenet;
pub mod ea;
pub mod emulator;
pub mod epic;
pub mod process;
pub mod steam;
pub mod ubisoft;

use crate::models::{Game, Platform};
use crate::metadata::GameMetadata;

pub fn scan_all_platforms() -> Vec<Game> {
    let mut all_games = Vec::new();

    // 1. Scan Steam
    all_games.extend(steam::SteamScanner::scan_installed_games());

    // 2. Scan Epic Games
    all_games.extend(epic::EpicScanner::scan_installed_games());

    // 3. Scan Ubisoft Connect
    all_games.extend(ubisoft::UbisoftScanner::scan_installed_games());

    // 4. Scan EA App / Origin
    all_games.extend(ea::EaScanner::scan_installed_games());

    // 5. Scan Blizzard / Battle.net
    all_games.extend(battlenet::BattleNetScanner::scan_installed_games());

    // 6. Scan Emulators & ROMs
    all_games.extend(emulator::EmulatorScanner::scan_installed_games());

    // 7. Add popular games to enrich grid visualization if not already installed locally
    for popular in get_popular_games() {
        if !all_games.iter().any(|g| g.id == popular.id || g.title.to_lowercase() == popular.title.to_lowercase()) {
            all_games.push(popular);
        }
    }

    // Index all cached covers from disk once for blazing fast O(1) matching
    let cached_covers_map = crate::artwork::get_all_cached_covers();
    for game in &mut all_games {
        if game.cover_image_path.is_none() {
            let safe_id = crate::artwork::sanitize_filename(&game.id);
            if let Some(path) = cached_covers_map.get(&safe_id).or_else(|| cached_covers_map.get(&game.id)) {
                game.set_cover_path(Some(path.clone()));
            }
        }
        if game.cover_uri.is_none() && game.cover_image_path.is_some() {
            if let Some(p) = &game.cover_image_path {
                game.cover_uri = Some(format!("file://{}", p.to_string_lossy().replace('\\', "/")));
            }
        }
    }

    // Sort all games alphabetically
    all_games.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));

    all_games
}

/// Popular games added to enrich the library grid visualization and test details/covers
pub fn get_popular_games() -> Vec<Game> {
    let mut games = Vec::new();

    let mut add_game = |id: &str,
                        title: &str,
                        platform: Platform,
                        launch_uri: &str,
                        size_gb: f64,
                        dev: &str,
                        publ: &str,
                        release: &str,
                        genres: &[&str],
                        summary: &str| {
        let mut game = Game::new(id, title, platform, launch_uri);
        game.install_size = Some((size_gb * 1024.0 * 1024.0 * 1024.0) as u64);

        let mut meta = GameMetadata::new(id, title);
        meta.developer = Some(dev.to_string());
        meta.publisher = Some(publ.to_string());
        meta.release_date = Some(release.to_string());
        meta.genres = genres.iter().map(|&g| g.to_string()).collect();
        meta.summary = Some(summary.to_string());
        game.metadata = Some(meta);

        games.push(game);
    };

    add_game(
        "1091500",
        "Cyberpunk 2077",
        Platform::Steam,
        "steam://rungameid/1091500",
        70.2,
        "CD PROJEKT RED",
        "CD PROJEKT RED",
        "10/dez./2020",
        &["RPG", "Ficção Científica", "Mundo Aberto"],
        "Cyberpunk 2077 é uma história de ação e aventura em mundo aberto ambientada em Night City, uma megalópole obcecada por poder, glamour e biomodificações.",
    );

    add_game(
        "1888930",
        "The Last of Us Part I",
        Platform::Steam,
        "steam://rungameid/1888930",
        79.4,
        "Naughty Dog LLC",
        "PlayStation Publishing LLC",
        "28/mar./2023",
        &["Ação", "Aventura", "História Rica"],
        "Conheça a história emocionante e os personagens inesquecíveis de The Last of Us, vencedor de mais de 200 prêmios de Jogo do Ano.",
    );

    add_game(
        "1245620",
        "Elden Ring",
        Platform::Steam,
        "steam://rungameid/1245620",
        60.0,
        "FromSoftware Inc.",
        "Bandai Namco Entertainment",
        "24/fev./2022",
        &["RPG de Ação", "Mundo Aberto", "Fantasia Sombria"],
        "O NOVO RPG DE AÇÃO E FANTASIA. Levante-se, Maculado, e seja guiado pela graça para portar o poder do Anel Prístino e se tornar um Lorde Prístino nas Terras Intermédias.",
    );

    add_game(
        "1174180",
        "Red Dead Redemption 2",
        Platform::Steam,
        "steam://rungameid/1174180",
        119.5,
        "Rockstar Games",
        "Rockstar Games",
        "05/dez./2019",
        &["Mundo Aberto", "História Rica", "Faroeste"],
        "Vencedor de mais de 175 prêmios de Jogo do Ano e avaliado com mais de 250 notas máximas, Red Dead Redemption 2 é uma história épica sobre honra e lealdade.",
    );

    add_game(
        "1593500",
        "God of War",
        Platform::Steam,
        "steam://rungameid/1593500",
        64.8,
        "Santa Monica Studio",
        "PlayStation Publishing LLC",
        "14/jan./2022",
        &["Ação", "Aventura", "Mitologia"],
        "Com a vingança contra os deuses do Olimpo no passado, Kratos agora vive no reino das divindades e monstros nórdicos.",
    );

    add_game(
        "1086940",
        "Baldur's Gate 3",
        Platform::Steam,
        "steam://rungameid/1086940",
        137.2,
        "Larian Studios",
        "Larian Studios",
        "03/ago./2023",
        &["RPG", "Escolhas Importam", "Estratégia"],
        "Reúna seu grupo e retorne aos Reinos Esquecidos em uma história de companheirismo e traição, sacrifício e sobrevivência, e a atração pelo poder absoluto.",
    );

    add_game(
        "GrandTheftAutoV",
        "Grand Theft Auto V",
        Platform::EpicGames,
        "com.epicgames.launcher://apps/GrandTheftAutoV?action=launch",
        108.4,
        "Rockstar North",
        "Rockstar Games",
        "14/abr./2015",
        &["Mundo Aberto", "Ação", "Multijogador"],
        "Quando um jovem malandro de rua, um ladrão de bancos aposentado e um psicopata aterrorizante se veem envolvidos no submundo do crime.",
    );

    add_game(
        "292030",
        "The Witcher 3: Wild Hunt",
        Platform::Steam,
        "steam://rungameid/292030",
        51.3,
        "CD PROJEKT RED",
        "CD PROJEKT RED",
        "18/mai./2015",
        &["RPG", "Mundo Aberto", "Fantasia"],
        "Você é Geralt de Rívia, um mercenário matador de monstros. Diante de você está um continente infestado de monstros e devastado pela guerra.",
    );

    add_game(
        "2195250",
        "EA SPORTS FC 24",
        Platform::Ea,
        "origin://launchgame/2195250",
        47.8,
        "EA Canada & EA Romania",
        "Electronic Arts",
        "29/set./2023",
        &["Esportes", "Futebol", "Competitivo"],
        "O EA SPORTS FC 24 é o início do futuro do futebol. Com o HyperMotionV, PlayStyles otimizados pela Opta e a Frostbite Engine aprimorada.",
    );

    add_game(
        "Fen",
        "Diablo IV",
        Platform::BattleNet,
        "battlenet://Fen",
        84.1,
        "Blizzard Entertainment",
        "Blizzard Entertainment",
        "05/jun./2023",
        &["RPG de Ação", "Hack and Slash", "Fantasia Sombria"],
        "A batalha sem fim entre o Paraíso Celestial e o Inferno Ardente continua enquanto o caos consome Santuário.",
    );

    add_game(
        "2358720",
        "Black Myth: Wukong",
        Platform::Steam,
        "steam://rungameid/2358720",
        128.4,
        "Game Science",
        "Game Science",
        "20/ago./2024",
        &["RPG de Ação", "Mitologia", "Aventura"],
        "Black Myth: Wukong é um RPG de ação inspirado na mitologia chinesa. A história é baseada em Jornada ao Oeste, um dos quatro grandes romances clássicos da literatura chinesa.",
    );

    add_game(
        "990080",
        "Hogwarts Legacy",
        Platform::Steam,
        "steam://rungameid/990080",
        73.1,
        "Avalanche Software",
        "Warner Bros. Games",
        "10/fev./2023",
        &["Mundo Aberto", "Magia", "RPG"],
        "Hogwarts Legacy é um RPG de ação imersivo e de mundo aberto ambientado no universo apresentado pela primeira vez nos livros de Harry Potter.",
    );

    add_game(
        "1817070",
        "Marvel's Spider-Man Remastered",
        Platform::Steam,
        "steam://rungameid/1817070",
        75.3,
        "Insomniac Games",
        "PlayStation Publishing LLC",
        "12/ago./2022",
        &["Super-Heróis", "Mundo Aberto", "Ação"],
        "Em Marvel's Spider-Man Remastered, os mundos de Peter Parker e do Homem-Aranha colidem em uma história original e repleta de ação.",
    );

    add_game(
        "2050650",
        "Resident Evil 4",
        Platform::Steam,
        "steam://rungameid/2050650",
        58.7,
        "CAPCOM Co., Ltd.",
        "CAPCOM Co., Ltd.",
        "24/mar./2023",
        &["Terror", "Ação", "Sobrevivência"],
        "A sobrevivência é apenas o começo. Seis anos após o desastre biológico em Raccoon City, o agente Leon S. Kennedy é enviado para resgatar a filha do presidente.",
    );

    add_game(
        "553850",
        "Helldivers 2",
        Platform::Steam,
        "steam://rungameid/553850",
        70.0,
        "Arrowhead Game Studios",
        "PlayStation Publishing LLC",
        "08/fev./2024",
        &["Tiro Cooperativo", "Ação", "Ficção Científica"],
        "Junte-se aos Helldivers e lute pela liberdade em uma galáxia hostil neste jogo de tiro em terceira pessoa cooperativo frenético.",
    );

    add_game(
        "1716740",
        "Starfield",
        Platform::Steam,
        "steam://rungameid/1716740",
        125.0,
        "Bethesda Game Studios",
        "Bethesda Softworks",
        "06/set./2023",
        &["RPG", "Espaço", "Mundo Aberto"],
        "Starfield é o primeiro universo novo em mais de 25 anos da Bethesda Game Studios, os premiados criadores de The Elder Scrolls V: Skyrim e Fallout 4.",
    );

    add_game(
        "1551360",
        "Forza Horizon 5",
        Platform::Steam,
        "steam://rungameid/1551360",
        130.6,
        "Playground Games",
        "Xbox Game Studios",
        "09/nov./2021",
        &["Corrida", "Mundo Aberto", "Simulador"],
        "Sua maior aventura no Horizon te espera! Lidere expedições de tirar o fôlego pelas paisagens vibrantes e em constante evolução do México em mundo aberto.",
    );

    add_game(
        "2215430",
        "Ghost of Tsushima DIRECTOR'S CUT",
        Platform::Steam,
        "steam://rungameid/2215430",
        75.0,
        "Sucker Punch Productions",
        "PlayStation Publishing LLC",
        "16/mai./2024",
        &["Mundo Aberto", "Samurais", "Ação"],
        "No final do século XIII, o império mongol devastou nações inteiras. A ilha de Tsushima é o único obstáculo entre o Japão e uma invasão massiva.",
    );

    add_game(
        "ac_mirage",
        "Assassin's Creed Mirage",
        Platform::Ubisoft,
        "uplay://launch/5059/0",
        40.0,
        "Ubisoft Montreal",
        "Ubisoft",
        "05/out./2023",
        &["Furtividade", "Ação", "História Rica"],
        "Conheça a história de Basim, um ladrão de rua astuto em busca de respostas e justiça pelas movimentadas ruas de Bagdá no século IX.",
    );

    add_game(
        "farcry6",
        "Far Cry 6",
        Platform::Ubisoft,
        "uplay://launch/5266/0",
        60.0,
        "Ubisoft Toronto",
        "Ubisoft",
        "07/out./2021",
        &["Mundo Aberto", "Tiro", "Ação"],
        "Bem-vindo a Yara, um paraíso tropical congelado no tempo. Antón Castillo, ditador de Yara, pretende restaurar sua nação custe o que custar.",
    );

    add_game(
        "alanwake2",
        "Alan Wake 2",
        Platform::EpicGames,
        "com.epicgames.launcher://apps/AlanWake2?action=launch",
        86.0,
        "Remedy Entertainment",
        "Epic Games Publishing",
        "27/out./2023",
        &["Terror Psicológico", "Mistério", "Sobrevivência"],
        "Uma série de assassinatos ritualísticos ameaça Bright Falls. Saga Anderson, uma talentosa agente do FBI, chega para investigar os assassinatos locais.",
    );

    add_game(
        "1850570",
        "Death Stranding Director's Cut",
        Platform::EpicGames,
        "com.epicgames.launcher://apps/DeathStranding?action=launch",
        80.0,
        "KOJIMA PRODUCTIONS",
        "505 Games",
        "30/mar./2022",
        &["Mundo Aberto", "Ficção Científica", "Exploração"],
        "Do lendário criador Hideo Kojima, surge uma experiência que desafia qualquer gênero, expandida e remasterizada para PC.",
    );

    add_game(
        "1145350",
        "Hades II",
        Platform::Steam,
        "steam://rungameid/1145350",
        10.0,
        "Supergiant Games",
        "Supergiant Games",
        "06/mai./2024",
        &["Roguelike", "Ação", "Mitologia"],
        "Batalhe além do Submundo usando feitiçaria sombria para enfrentar o Titã do Tempo na sequência do aclamado dungeon crawler rogue-like.",
    );

    add_game(
        "overwatch2",
        "Overwatch 2",
        Platform::BattleNet,
        "battlenet://Pro",
        50.0,
        "Blizzard Entertainment",
        "Blizzard Entertainment",
        "10/ago./2023",
        &["Tiro em Primeira Pessoa", "Competitivo", "Heróis"],
        "Overwatch 2 é um jogo de tiro em equipe gratuito e em constante evolução, ambientado em um futuro otimista, com batalhas 5v5 definitivas.",
    );

    add_game(
        "zelda_totk",
        "The Legend of Zelda: Tears of the Kingdom",
        Platform::Emulator,
        "gank://emulator/zelda_totk",
        18.2,
        "Nintendo",
        "Nintendo",
        "12/mai./2023",
        &["Aventura", "Mundo Aberto", "Quebra-Cabeças"],
        "Uma aventura épica pelas terras e céus de Hyrule aguarda nesta sequência de The Legend of Zelda: Breath of the Wild.",
    );

    add_game(
        "mario_odyssey",
        "Super Mario Odyssey",
        Platform::Emulator,
        "gank://emulator/mario_odyssey",
        5.7,
        "Nintendo",
        "Nintendo",
        "27/out./2017",
        &["Plataforma", "Aventura", "Exploração 3D"],
        "Embarque em uma jornada cativante pelo mundo afora com Mario e seu novo aliado Cappy em uma enorme aventura 3D estilo sandbox.",
    );

    games
}
