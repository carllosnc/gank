# AGENTS.md — Guia e Diretrizes para Agentes de IA no Gank

Este documento define a arquitetura, padrões de código, regras de design e o roadmap do projeto **Gank**. Qualquer agente de IA ou desenvolvedor pareado deve seguir rigorosamente estas diretrizes ao propor, modificar ou refatorar código neste repositório.

---

## 🎮 Visão Geral do Projeto

**Gank** é um agregador e inicializador desktop de jogos ultrarrápido, minimalista e elegante, desenvolvido em **Rust**, utilizando **egui** e **wgpu**.

- **Objetivo Principal:** Centralizar bibliotecas de jogos fragmentadas (Steam, Epic Games, EA App, Ubisoft Connect, Battle.net, GOG, Xbox e Emuladores) em uma única interface cinematográfica e fluida, sem a sobrecarga de múltiplos launchers abertos em segundo plano.
- **Diferencial Competitivo:** Consumo de CPU inferior a 1% em idle, inicialização instantânea em milissegundos, visual imersivo com shaders WGPU em tempo real e arquitetura com zero alocações por frame de renderização.

---

## 🏛️ Arquitetura e Estrutura de Pastas

```text
gank/
├── assets/
│   ├── fonts/           # Inter-Regular e Inter-SemiBold (.otf)
│   └── icons/           # Ícones oficiais SVG e PNG de alta resolução
├── src/
│   ├── main.rs          # Ponto de entrada, configuração do eframe e pipeline WGPU
│   ├── models.rs        # Estruturas centrais (Game, Platform, LaunchCommand)
│   ├── config.rs        # Configuração do app e chaves de API persistidas em JSON
│   ├── artwork.rs       # Cache local de capas, conversão ffmpeg de capas animadas e thumbnails
│   ├── metadata/        # Enriquecimento assíncrono de metadados
│   │   ├── mod.rs       # Gerenciador e cache em disco de metadados
│   │   ├── rawg.rs      # Integração com API RAWG
│   │   ├── steam_store.rs # Web scraping leve da loja da Steam
│   │   └── steamgrid.rs # Integração com SteamGridDB
│   ├── scanners/        # Módulos de detecção local de plataformas instaladas
│   │   ├── mod.rs       # Agregador multi-plataforma e mock de jogos populares
│   │   ├── steam.rs     # Parser de manifests VDF (.acf) e libraryfolders
│   │   ├── epic.rs      # Parser de manifests da Epic Games (.item)
│   │   ├── ubisoft.rs   # Leitor do registro e manifests Ubisoft Connect
│   │   ├── ea.rs        # Leitor de manifestos e registro EA App / Origin
│   │   ├── battlenet.rs # Parser do banco de dados Battle.net (.db / registry)
│   │   ├── emulator.rs  # Varredor de ROMs e emuladores (RetroArch, RPCS3, etc.)
│   │   └── process.rs   # Utilitários de processos e execução segura
│   └── ui/              # Camada de apresentação e componentes egui
│       ├── mod.rs       # Exportações do módulo de UI
│       ├── app.rs       # Loop de renderização principal (GankApp) e detalhes do jogo
│       ├── aurora_shader.rs # Shaders WGSL de poeira cósmica e mesh gradient
│       ├── components.rs# Pílulas de filtro, botões de ação e cards padronizados
│       ├── icons.rs     # Imagens estáticas e glyphs Phosphor
│       └── theme.rs     # Tokens de design (cores Tailwind Blue, tipografia, raios)
├── AGENTS.md            # Este arquivo de governança
├── ROADMAP.md           # Roadmap detalhado para a versão v0.1.0 e além
├── Cargo.toml           # Dependências e perfis de otimização de release
└── README.md            # Documentação pública do repositório
```

---

## ⚡ Regras de Ouro de Performance (Anti-Regressão)

1. **Zero-Allocation no Loop Principal (`update`):**
   - **Proibido** clonar coleções inteiras de `Game` ou `String`s a cada frame (16ms).
   - Use fatias e referências (`Vec<&Game>`). Clonagens de `Game` só devem ocorrer pontualmente quando o usuário clica para abrir a página de detalhes ou disparar o executável.
2. **I/O Não Bloqueante (Thread Isolation):**
   - **Nunca** realize operações de rede (`ureq`), leituras extensas de disco (`fs::read_dir`) ou decodificação pesada de imagens dentro do método `update` da UI.
   - Todo I/O deve ser despachado via canais `std::sync::mpsc` para worker threads dedicadas.
3. **Caches em Memória e `OnceLock`:**
   - O caminho do diretório de capas deve ser resolvido uma única vez com `std::sync::OnceLock`.
   - Listas de capas animadas e mapeamentos de capas em disco devem ser consultados em `HashSet` / `HashMap` mantidos em memória, evitando syscalls do Windows a cada frame.
4. **Focus Throttling:**
   - O aplicativo roda a 60 FPS com VSync (`Fifo`) enquanto a janela estiver em foco.
   - Quando a janela perde o foco (ex.: usuário está em um jogo ou navegando), a taxa de repintura deve ser reduzida para 15-30 FPS (66ms) para poupar CPU/GPU.
5. **Viewport Culling Obrigatório:**
   - Cards e posters fora da área visível (`!ui.is_rect_visible(rect)`) devem abortar a pintura imediatamente para evitar trabalho inútil da GPU.

---

## 🎨 Design System e Diretrizes Visuais

- **Paleta de Cores:**
  - Base: Midnight Velvet escuro (`#080b12` a `#020408`) via Mesh Gradient WGPU.
  - Acento Primário: **Tailwind Blue** (`BLUE_500`: `#3b82f6`, `BLUE_600`: `#2563eb`, `BLUE_700`: `#1d4ed8`).
  - Superfícies: Vidro translúcido escuro (`rgba(18, 22, 30, alpha)`).
  - Pílulas e Seletores: Borda sutil de 1px azul no estado ativo, sem linhas divisórias duras (*no separator lines*).
- **Tipografia:**
  - Fonte Proporcional: **Inter** (Regular para corpo, SemiBold para títulos e botões).
  - Ícones: **Phosphor Icons** integrados nativamente nas famílias de fonte do egui + Ícones oficiais de plataformas vetoriais/rasterizados em alta resolução.
- **Proporção de Capas:**
  - Padrão oficial 2:3 vertical (box art padrão Steam/GOG: 600x900 / 300x450).
  - Arredondamento suave de 8.0px nos cards da biblioteca e 12.0px no poster hero da tela de detalhes.

---

## 🗺️ Roadmap de Lançamento: Versão v0.1.0

### Status Atual (95% Concluído):
- [x] Scanners para 6 plataformas (Steam, Epic, Ubisoft, EA, Battle.net, Emuladores).
- [x] Auto-inicialização silenciosa de launchers em segundo plano (Steam, Epic, EA, Ubisoft).
- [x] Shader WGSL nativo com poeira estelar flutuante e mesh gradient orgânico.
- [x] Tela cinematográfica de detalhes do jogo (Poster expandido, sinopse, desenvolvedora, gênero, tamanho em disco).
- [x] Suporte a capas personalizadas com drag-and-drop e colagem de URL (Ctrl+V).
- [x] Capas animadas com conversão de vídeo loop (ffmpeg) para frames 2:3.
- [x] Otimizações de Zero-Allocation e Throttling de foco com consumo <0.4% CPU em idle.
- [x] Empacotamento e Ícone do Executável Windows (.ico nos recursos do executável via build.rs/winres e janela via with_icon).
- [x] Configuração do Repositório Remoto no GitHub (https://github.com/carllosnc/gank).
