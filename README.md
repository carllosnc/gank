# Kiro 🎮

Um agregador e inicializador rápido de atalhos de jogos para desktop, desenvolvido em **Rust** com foco estrito em **desempenho**, **baixo consumo de recursos** e **inicialização instantânea**.

O objetivo do Kiro é centralizar todos os seus jogos instalados (Steam, Epic Games, GOG, Xbox, etc.) em uma interface única, rápida e reativa, eliminando a necessidade de manter múltiplos inicializadores abertos consumindo memória em segundo plano.

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
kiro/
├── src/
│   ├── models.rs        # Estruturas de dados (Game, Platform, LaunchCommand)
│   ├── scanners/        # Módulos de detecção de jogos locais
│   │   ├── mod.rs
│   │   └── steam.rs     # Scanner para instalações da Steam
│   ├── ui/              # Componentes de interface e estado do egui
│   │   ├── mod.rs
│   │   └── app.rs
│   └── main.rs          # Ponto de entrada e configuração do eframe
├── Cargo.toml
├── LICENSE
└── README.md
```

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

O binário final compilado estará disponível em `target/release/kiro.exe`.

---

## 📄 Licença

Distribuído sob a licença MIT. Veja [`LICENSE`](LICENSE) para mais detalhes.
