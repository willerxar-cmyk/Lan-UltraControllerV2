# AGENTS.md - Trabalhando no Lan Mouse (Português)

Este documento ajuda os agentes a entender a codebase do Lan Mouse para desenvolvimento efetivo.

## Visão Geral do Projeto

Lan Mouse é um software KVM switch (compartilhamento de mouse e teclado em LAN) multi-plataforma escrito em Rust. Cada instância pode enviar e receber eventos de entrada.

**Características principais**:
- Workspace em Rust com múltiplos crates
- Async/await com runtime tokio
- Suporte multi-plataforma (Linux, Windows, macOS, X11, Wayland)
- Múltiplas implementações de backends para captura/emulação de entrada
- Frontend GTK4/libadwaita (opcional)
- Comunicação de rede criptografada com DTLS

## Estrutura do Workspace

```
lan-mouse/              # Crate principal (serviço + aplicação)
├── src/                # Lógica do serviço principal
├── Cargo.toml          # Raiz do workspace + crate principal
└── build.rs            # Script de build do shadow-rs

input-capture/          # Backends de captura de entrada (layer-shell, libei, Windows, macOS, X11)
├── src/
└── Cargo.toml          # Features específicos da plataforma

input-emulation/        # Backends de emulação de entrada (wlroots, libei, Windows, macOS, X11, xdp)
├── src/
└── Cargo.toml          # Features específicos da plataforma

input-event/            # Tipos de eventos compartilhados e definições de scancode
├── src/
│   ├── lib.rs          # Enums de eventos (PointerEvent, KeyboardEvent, Event)
│   ├── scancode.rs     # Mapeamentos de scancode Linux
│   └── error.rs        # Tipos de erro
└── Cargo.toml

lan-mouse-ipc/          # Biblioteca de IPC para comunicação serviço-frontend
├── src/
└── Cargo.toml

lan-mouse-cli/          # Interface de linha de comando
├── src/
└── Cargo.toml

lan-mouse-gtk/          # Frontend GTK4/libadwaita (feature opcional)
├── src/
├── resources/
│   └── de.feschber.LanMouse.svg
├── build.rs            # glib-build-tools
└── Cargo.toml

lan-mouse-proto/        # Definições de protocolo e serialização de eventos
├── src/
│   └── lib.rs          # Enums ProtoEvent, Position, codificação/decodificação
└── Cargo.toml

screen-share/            # [NOVO] Módulo de compartilhamento de tela
├── src/
│   ├── lib.rs          # Tipos principais e interfaces
│   ├── capture.rs      # Backends de captura de tela
│   ├── display.rs      # Backends de exibição de tela
│   ├── encoder.rs      # Codificador AV1
│   ├── protocol.rs      # Protocolo de streaming de vídeo
│   └── error.rs        # Tipos de erro
└── Cargo.toml
```

## Comandos Essenciais

### Build
```bash
# Build padrão (inclui features padrão)
cargo build

# Build release com otimizações
cargo build --release

# Compilação condicional (ex: apenas suporte Sway)
cargo build --no-default-features --features layer_shell_capture,wlroots_emulation

# Builds específicos de plataforma
cargo build --features gtk           # Incluir frontend GTK
cargo build --features x11_capture     # Incluir captura X11
cargo build --features screen_share    # Incluir compartilhamento de tela
```

### Testes
```bash
# Executar todos os testes
cargo test

# Testes com saída detalhada
cargo test --verbose

# Teste específico
cargo test nome_do_teste
```

### Linting e Formatação
```bash
# Verificar formatação
cargo fmt --check

# Aplicar formatação
cargo fmt

# Executar clippy (warnings causam falha no CI)
cargo clippy --all-features --all-targets -- --deny warnings

# Verificar advisories de segurança (requer cargo-deny)
cargo deny check advisories
```

### Executar a Aplicação
```bash
# Executar com frontend GTK (padrão)
./target/debug/lan-mouse

# Executar em modo daemon
./target/debug/lan-mouse daemon

# Executar interface CLI
./target/debug/lan-mouse cli help
```

## Estilo de Código e Convenções

### Formatação
- **Style edition**: 2024
- **Indentação**: 4 espaços
- **Largura máxima de linha**: 100 caracteres
- Definido em `.rustfmt.toml`

### Tratamento de Erros
- Usa crate `thiserror` para tipos de erro
- Tipos de erro derivam `Debug` e `Error`
- Padrão comum:
```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MeuErro {
    #[error("falha ao fazer algo: {0}")]
    AlgoFalhou(String),

    #[error(transparent)]
    Io(#[from] io::Error),
}
```

### Padrões Async
- Usa runtime `tokio` com features: `io-util`, `io-std`, `macros`, `net`, `process`, `rt`, `sync`, `signal`
- Funções async usam `async fn` e `.await`
- Processamento de streams com `futures::StreamExt`

### Configuração
- Usa `toml` para arquivos de configuração
- Localização do config: `$XDG_CONFIG_HOME/lan-mouse/config.toml` (Linux/macOS) ou `%LOCALAPPDATA%\lan-mouse\` (Windows)
- Structs de configuração usam `serde` `Serialize`/`Deserialize`
- Veja `config.example.toml` para configuração de exemplo

### Gerenciamento de Clientes
- Usa `slab::Slab` para armazenamento de clientes (`Rc<RefCell<Slab<...>>>`)
- Clientes identificados por `ClientHandle` (usize cast)
- Padrão ClientManager em `src/client.rs`

### Nomes de Scancode
- Símbolos de tecla seguem nomes em `input-event/src/scancode.rs`
- Exemplo: `KeyA`, `KeyS`, `KeyLeftShift`, `KeyLeftCtrl`, `KeyLeftMeta`
- Usado em configuração `release_bind`

## Arquitetura e Padrões

### Fluxo de Eventos
```
Backend (Wayland/X11/Windows/macOS)
    ↓ InputEvent
Capture → Emitter (DTLS over UDP)
    ↓ Rede
Receiver → Dispatcher
    ↓ InputEvent
Backend (Wayland/X11/Windows/macOS)
```

### Estado de Dispositivos
- Cada dispositivo está em **ativo** (enviando eventos) ou **inativo** (recebendo eventos)
- Previne loops de feedback - quando controlando outro dispositivo, o dispositivo controlado não envia eventos de volta
- Gerenciado através de máquina de estados em `ClientManager`

### Backends

**Backends de Captura de Entrada**:
- `layer_shell`: Compositores Wayland com suporte layer-shell (Sway, Wayfire, etc.)
- `libei`: GNOME >= 45, KDE Plasma >= 6.1
- `windows`: Windows
- `macos`: macOS
- `x11`: X11 (WIP)

**Backends de Emulação de Entrada**:
- `wlroots`: Compositores wlroots via protocolo virtual-pointer
- `libei`: GNOME >= 45, KDE Plasma >= 6.1
- `remote_desktop_portal`: freedesktop remote desktop portal
- `windows`: Windows
- `macos`: macOS
- `x11`: X11

**[NOVO] Módulo Screen Share**:
- `screen-share`: Compartilhamento de tela com alternância rápida
  - Captura de tela multi-plataforma
  - Codificação AV1 de vídeo
  - Exibição de tela remota
  - Hotkeys: Ctrl+Shift+Up/Down

### Protocolo de Rede
- UDP para transmissão de eventos (baixa latência)
- TCP para requisições de controle
- Criptografia DTLS usando `webrtc-dtls` e `rustls`
- Porta padrão: 4242 (configurável)

## Compilação Condicional

O projeto usa features do Cargo extensivamente para código específico de plataforma:

**Features padrão** (veja `Cargo.toml`):
```toml
default = [
    "gtk",
    "layer_shell_capture",
    "x11_capture",
    "libei_capture",
    "wlroots_emulation",
    "libei_emulation",
    "rdp_emulation",
    "x11_emulation",
    "screen_share",  # [NOVO]
]
```

**Dependências específicas de plataforma**:
- Use `#[cfg(unix)]`, `#[cfg(windows)]`, `#[cfg(target_os = "macos")]` para gate de plataforma
- Use `#[cfg(feature = "feature_name")]` para gate de features
- Veja `input-capture/Cargo.toml` e `input-emulation/Cargo.toml` para exemplos

## Testes

### Organização de Testes
- Testes de unidade em `src/` junto com implementação de arquivos
- Testes de integração em `src/capture_test.rs` e `src/emulation_test.rs`
- Comandos de teste expostos via CLI: `lan-mouse test-emulation` e `lan-mouse test-capture`

### Executando Testes para Desenvolvimento
```bash
# Testar funcionalidade específica
cargo test --package lan-mouse

# Testar backend específico
cargo test --package input-capture
```

## Problemas Comuns e Soluções

### 1. Seleção de Backend
Backends são selecionados automaticamente baseados no ambiente de desktop ativo. Ao trabalhar em um backend específico, pode ser necessário sobrescrever:
```bash
lan-mouse --capture-backend=layer-shell --emulation-backend=wlroots
```

### 2. Dependências GTK no Windows
Compilar GTK do zero no Windows é complexo. Use a abordagem gvsbuild documentada em README.md ou desabilite o GTK com `--no-default-features`.

### 3. Suporte libei
O suporte libei requer GNOME recente (>= 45) ou KDE Plasma (>= 6.1). Certifique-se de que seu ambiente de teste atende aos requisitos.

### 4. Teclas de Modificador no wlroots
Compositores wlroots sem suporte libei no lado receptor não tratam eventos de modificador adequadamente quando o dispositivo de envio NÃO está usando o backend layer-shell.

### 5. Configuração do Wayfire
Ao usar Wayfire, certifique-se de adicionar `shortcuts-inhibit` à lista de plugins no seu config do wayfire, caso contrário a captura de entrada não funcionará.

### 6. Cursor de Mouse do Windows
Ao enviar entrada para Windows, o cursor do mouse será invisível se não houver um mouse real conectado à máquina.

### 7. Limitações do X11
O X11 atualmente tem suporte apenas para emulação de entrada (pode ser usado apenas como receptor, não como emissor). Captura de entrada é WIP.

### 8. Loops de Feedback
Ao implementar novas funcionalidades, considere sempre o estado do dispositivo (ativo/inativo) para prevenir loops de eventos de feedback. A lógica no nível de serviço deve garantir que dispositivos controlados não transmitam eventos de volta.

### 9. Gerenciamento de Certificados
O aplicativo gera certificados DTLS automaticamente. Certificados são armazenados em `~/.config/lan-mouse/lan-mouse.pem` (Unix) ou `%LOCALAPPDATA%\lan-mouse\lan-mouse.pem` (Windows). Fingerprints são usados para autorização.

### 10. Logging
Configure o logging com a variável de ambiente `LAN_MOUSE_LOG_LEVEL`:
```bash
export LAN_MOUSE_LOG_LEVEL=debug
./target/debug/lan-mouse
```

## Desenvolvimento Específico de Plataforma

### Linux
- Instalar dependências: `libadwaita-1-dev`, `libgtk-4-dev`, `libx11-dev`, `libxtst-dev` (Debian/Ubuntu)
- Para NixOS: usar `nix-shell` ou `nix develop`

### Windows
- Requer compilar GTK4 do zero via gvsbuild
- Adicionar `C:\gtk-build\gtk\x64\release\bin` ao PATH
- Ou usar `--no-default-features` para desabilitar GTK

### macOS
- Instalar dependências: `brew install libadwaita pkg-config imagemagick`
- Para app bundle: `cargo install cargo-bundle`, então `cargo bundle`
- Executar `scripts/copy-macos-dylib.sh` para copiar bibliotecas dinâmicas

## CI/CD

GitHub Actions (`.github/workflows/rust.yml`) executa:
- Build em Linux, Windows, macOS (Intel e ARM)
- Testes: `cargo test --verbose`
- Verificação de formatação: `cargo fmt --check`
- Clippy: `cargo clippy --all-features --all-targets -- --deny warnings`

Todos os checks devem passar antes de fazer merge.

## Adicionando Novas Funcionalidades

### Novo Backend
1. Implementar em `input-capture` ou `input-emulation` crate
2. Adicionar feature flag ao respectivo `Cargo.toml`
3. Adicionar dependências específicas de plataforma se necessário
4. Atualizar lógica de seleção de backend em `config.rs`
5. Adicionar feature às features padrão no `Cargo.toml` raiz se apropriado

### Novo Tipo de Evento
1. Adicionar a `input-event/src/lib.rs` (enum Event)
2. Atualizar protocolo em `lan-mouse-proto/src/lib.rs`
3. Implementar serialização/deserialização
4. Atualizar todos os backends para tratar o novo tipo de evento

### Extensão de IPC
1. Adicionar tipo de evento/requisição em `lan-mouse-ipc/src/lib.rs`
2. Tratar em `src/service.rs` (lado servidor) e frontend (lado cliente)

### [NOVO] Módulo Screen Share
O módulo screen-share já foi criado com:
- Backends de captura de tela (placeholder, pronto para implementação)
- Codificador AV1 (placeholder, pronto para implementação)
- Protocolo de streaming de vídeo
- Backends de exibição (placeholder, pronto para implementação)
- Integração com service existente
- Hotkeys Ctrl+Shift+Up/Down

## Referência de Arquivos Principais

- `src/main.rs`: Ponto de entrada da aplicação, roteamento de comandos
- `src/service.rs`: Implementação do serviço principal
- `src/config.rs`: Parsing de configuração e tratamento de argumentos CLI
- `src/client.rs`: Gerenciamento de clientes
- `src/capture.rs`: Wrapper de captura de entrada
- `src/emulation.rs`: Wrapper de emulação de entrada
- `src/listen.rs`: Listener UDP para eventos de entrada
- `src/connect.rs`: Gerenciamento de conexões
- `src/crypto.rs`: Tratamento de certificados DTLS
- `src/dns.rs`: Resolução DNS para hostnames
- `src/screen_share.rs`: [NOVO] Gerenciador de compartilhamento de tela
- `input-event/src/scancode.rs`: Mapeamentos de scancode Linux
- `lan-mouse-ipc/src/lib.rs`: Tipos de IPC e comunicação
- `lan-mouse-proto/src/lib.rs`: Protocolo de serialização de eventos
- `screen-share/src/`: [NOVO] Módulo de screen share
