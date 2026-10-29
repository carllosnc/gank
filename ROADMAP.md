# Roadmap do Gank

Este documento descreve o plano de desenvolvimento do **Gank**, detalhando os marcos da versão inicial **v0.1.0** e as futuras evoluções planejadas.

---

## 🚀 Versão v0.1.0 — "Genesis Release" (Foco: Estabilidade e Funcionalidades Essenciais)

O objetivo da versão **v0.1.0** é entregar um agregador de jogos de alto desempenho, visualmente deslumbrante, pronto para uso diário em sistemas Windows.

### 📋 Checklist de Entrega

#### 1. Core & Scanners (Concluído ✅)
- [x] Detecção de jogos da **Steam** (leitura direta de arquivos `.acf` e bibliotecas extras).
- [x] Detecção de jogos da **Epic Games Store** (leitura de manifestos `.item`).
- [x] Detecção de jogos da **Ubisoft Connect** (leitura de registro e manifests).
- [x] Detecção de jogos do **EA App / Origin** (registro e manifestos locais).
- [x] Detecção de jogos da **Blizzard / Battle.net** (leitura de banco local e chaves de registro).
- [x] Detecção de **Emuladores e ROMs** (RetroArch, limpeza de títulos e caminhos de execução).
- [x] Inicialização silenciosa de launchers em segundo plano (Steam, Epic, EA, Ubisoft).
- [x] Resolução de tamanho de instalação e formatação amigável (GB/MB).

#### 2. Interface do Usuário & Experiência (Concluído ✅)
- [x] Grade de jogos dinâmica e responsiva com proporção vertical 2:3.
- [x] Fundo interativo em WGPU nativo com Mesh Gradient líquido e partículas cósmicas.
- [x] Efeito cascata suave na inicialização do grid de jogos.
- [x] Pílulas de filtro com ícones oficiais de alta resolução das plataformas.
- [x] Tela cinematográfica de detalhes do jogo (Poster hero ampliado, metadados, sinopse e tags).
- [x] Alteração de capas com arrastar-e-soltar (Drag-and-drop) e colar link (Ctrl+V).
- [x] Suporte experimental a capas animadas (vídeos e GIFs convertidos via ffmpeg).
- [x] Design System Showcase com tokens centralizados (Tailwind Blue, Inter Font, Phosphor Icons).

#### 3. Performance & Eficiência (Concluído ✅)
- [x] Consumo de CPU < 0.4% em repouso a 60 FPS com VSync.
- [x] *Zero-Allocation* no loop de renderização (filtragem por referências emprestadas).
- [x] *Focus Throttling*: taxa de quadros reduzida para ~15 FPS quando a janela não está focada.
- [x] *Viewport Culling*: cards fora da tela não consomem processamento de desenho.
- [x] Cache persistente de capas e metadados em JSON e JPEG locais.
- [x] Suite de testes unitários automatizados (14 testes passando).

#### 4. Metas Finais para Fechar a v0.1.0 ⏳
- [ ] **Empacotamento com Ícone de Aplicativo Windows:**
  - Criar script de compilação com `winres` / `build.rs` para embutir o ícone `.ico` oficial e manifesto Windows no `.exe`.
- [ ] **Repositório Remoto & CI:**
  - Publicar repositório no GitHub e criar GitHub Actions para testes e compilação automatizada da release.

---

## 🔮 Futuras Versões

### Versão v0.2.0 — "Deep Integration"
- **Suporte a GOG Galaxy e Xbox Game Pass (PC):**
  - Leitura do banco SQLite do GOG e manifestos de pacotes AppX da Microsoft Store.
- **Rastreamento de Tempo de Jogo Nativo:**
  - Monitoramento em segundo plano do processo do jogo para calcular horas jogadas no Gank.
- **Categorias e Tags Customizadas:**
  - Criação de coleções personalizadas pelo usuário ("Favoritos", "Zerar este mês", "Multiplayer").

### Versão v0.3.0 — "Deck & Controller Experience"
- **Modo Big Picture / Gamepad:**
  - Navegação completa por controle (Xbox, PlayStation, DualSense) com foco visual destacado.
- **Suporte Cross-Platform (Linux / Steam Deck):**
  - Adaptação das chamadas de sistema e caminhos do Proton/Wine para rodar nativamente no SteamOS/Linux.
