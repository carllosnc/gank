# Gank 🎮

Um agregador e inicializador rápido de atalhos de jogos para desktop, desenvolvido em **Rust** com foco estrito em **desempenho**, **baixo consumo de recursos** e **inicialização instantânea**.

O objetivo do Gank é centralizar todos os seus jogos instalados (Steam, Epic Games, GOG, Xbox, etc.) em uma interface única, rápida e reativa, eliminando a necessidade de manter múltiplos inicializadores abertos consumindo memória em segundo plano.

---

## ⚡ Princípios de Performance

- **Zero Sobrecarga em Repouso**: Interface reativa construída com `egui` e renderizada via `wgpu`. O consumo de CPU cai para 0% quando o aplicativo está em repouso.
- **Detecção Leve de Jogos**: Varredura direta através de manifestos locais e registro do sistema (sem requisições lentas de rede ou inicialização pesada de APIs no startup).
- **Consumo de Memória Reduzido**: Binário nativo compacto e cache eficiente de metadados e capas.
- **Disparo Direto**: Execução rápida dos jogos via URIs de protocolo ou executáveis diretos.

---

## 🛠️ Stack Tecnológica

- **Linguagem**: [Rust](https://www.rust-lang.org/) (Edição 2021)
- **GUI & Renderização**: [egui](https://github.com/emilk/egui) / [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) com backend [wgpu](https://wgpu.rs/)
- **Integração com Sistema Operacional**: Leitura de registro do Windows (`winreg`) e manipulação de processos do sistema

---

## 📂 Estrutura do Projeto

```text
gank/
├── assets/              # Fontes Inter e ícones oficiais de alta resolução
├── src/
│   ├── main.rs          # Ponto de entrada e configuração do pipeline WGPU
│   ├── models.rs        # Estruturas centrais de dados (Game, Platform)
│   ├── config.rs        # Configurações do app e chaves de API persistidas
│   ├── artwork.rs       # Cache de capas locais e processamento ffmpeg
│   ├── metadata/        # Enriquecimento assíncrono (RAWG, Steam Store, SteamGridDB)
│   ├── scanners/        # Detectores para Steam, Epic, EA, Ubisoft, Battle.net, Emuladores
│   └── ui/              # Componentes de interface, shaders WGPU, design system e telas
├── AGENTS.md            # Guia de arquitetura e regras para agentes de IA
├── ROADMAP.md           # Roadmap detalhado para a versão v0.1.0 e marcos futuros
├── Cargo.toml
├── LICENSE
└── README.md
```

---

## 🗺️ Roadmap & Diretrizes

- Para consultar o status das funcionalidades e as metas de lançamento, veja o [`ROADMAP.md`](ROADMAP.md).
- Para diretrizes de desenvolvimento, regras de performance e padrões de arquitetura, consulte o [`AGENTS.md`](AGENTS.md).

---

## 🚀 Como Executar

### Pré-requisitos
- [Rust & Cargo](https://rustup.rs/) instalados.

### Executando em desenvolvimento
```powershell
cargo run
```

### Compilando a versão otimizada
```powershell
cargo build --release
```

O binário final compilado estará disponível em `target/release/gank.exe`.

---

## 📄 Licença

Distribuído sob a licença MIT. Veja [`LICENSE`](LICENSE) para mais detalhes.
