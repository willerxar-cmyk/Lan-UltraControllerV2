# Script para configurar Git e fazer upload para GitHub

# Configurações
$GIT_EMAIL = "willerxar@gmail.com"
$GIT_USER = "willerxar"
$REPO_NAME = "lan-mouse-screen-share"
$REPO_DESCRIPTION = "Lan Mouse with KVM Switch Screen Sharing - Complete implementation with AV1 encoding"

Write-Host "🚀 Lan-Mouse Screen Share - Upload para GitHub" -ForegroundColor Cyan
Write-Host ""

# 1. Verificar se Git está instalado
Write-Host "📦 Verificando instalação do Git..." -ForegroundColor Yellow
$gitVersion = git --version 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Git não está instalado! Por favor, instale o Git primeiro." -ForegroundColor Red
    Write-Host "   Download: https://git-scm.com/download/win" -ForegroundColor White
    exit 1
}
Write-Host "✅ Git encontrado: $gitVersion" -ForegroundColor Green
Write-Host ""

# 2. Verificar se já é um repositório Git
if (Test-Path ".git") {
    Write-Host "⚠️  Este diretório já é um repositório Git" -ForegroundColor Yellow
    $choice = Read-Host "   Deseja reconfigurar? (s/N)"
    if ($choice -ne "s" -and $choice -ne "S") {
        Write-Host "   Operação cancelada." -ForegroundColor Yellow
        exit 0
    }
    Write-Host "   Limpando configuração Git..." -ForegroundColor Gray
    Remove-Item -Recurse -Force .git
}

# 3. Inicializar repositório Git
Write-Host "📝 Inicializando repositório Git..." -ForegroundColor Yellow
git init
git config user.name "$GIT_USER"
git config user.email "$GIT_EMAIL"
Write-Host "✅ Repositório Git inicializado" -ForegroundColor Green
Write-Host ""

# 4. Criar .gitignore
Write-Host "📝 Criando .gitignore..." -ForegroundColor Yellow
$gitignoreContent = @"
# Build outputs
target/
*.o
*.so
*.dylib
*.dll
*.exe

# IDEs
.vscode/
.idea/
*.swp
*.swo
*~

# OS
.DS_Store
Thumbs.db

# Test
/tmp/
"@
Set-Content -Path ".gitignore" -Value $gitignoreContent -Encoding UTF8
git add .gitignore
Write-Host "✅ .gitignore criado" -ForegroundColor Green
Write-Host ""

# 5. Adicionar todos os arquivos
Write-Host "📂 Adicionando arquivos ao Git..." -ForegroundColor Yellow
git add .
$addedCount = git diff --cached --name-only | Measure-Object -Line | Select-Object -ExpandProperty Lines
Write-Host "✅ $($addedCount.Count) arquivos adicionados" -ForegroundColor Green
Write-Host ""

# 6. Criar commit inicial
$commitMessage = "feat(lan-mouse): Add complete screen sharing module

- Implemented screen-share crate with AV1 encoding support
- Added ScreenCapture and ScreenDisplay backends
- Integrated ScreenShareManager with service
- Added screen sharing configuration (enable, fps, quality)
- Implemented hotkeys for screen switching (Ctrl+Shift+Up/Down)
- Complete documentation in Portuguese
- Optimized for maximum efficiency (zero-copy, inlining, etc.)

This allows Lan Mouse to function as a complete KVM switch,
sharing both input and screen between multiple computers."
Write-Host "💾 Criando commit inicial..." -ForegroundColor Yellow
git commit -m $commitMessage
Write-Host "✅ Commit criado" -ForegroundColor Green
Write-Host ""

# 7. Mostrar instruções para GitHub
Write-Host "🌐 Próximos passos (criar repositório no GitHub):" -ForegroundColor Cyan
Write-Host ""
Write-Host "1. Acesse: https://github.com/new" -ForegroundColor White
Write-Host "2. Nome do repositório: $REPO_NAME" -ForegroundColor Yellow
Write-Host "3. Descrição: $REPO_DESCRIPTION" -ForegroundColor Gray
Write-Host "4. Marque como: ☑️ Public" -ForegroundColor Yellow
Write-Host "5. Clique em: 'Create repository'" -ForegroundColor White
Write-Host ""
Write-Host "6. Depois de criar, execute:" -ForegroundColor Yellow
Write-Host ""
Write-Host "   git remote add origin https://github.com/$GIT_USER/$REPO_NAME.git" -ForegroundColor Cyan
Write-Host "   git branch -M main" -ForegroundColor Cyan
Write-Host "   git push -u origin main" -ForegroundColor Cyan
Write-Host ""

# 8. Oferecer para criar repositório automaticamente com GitHub CLI
Write-Host "💡 Deseja criar o repositório automaticamente usando GitHub CLI?" -ForegroundColor Cyan
Write-Host "   Se sim, instale: https://cli.github.com/" -ForegroundColor Gray
Write-Host "   Depois, execute: gh repo create --public --source=. --remote=origin --push" -ForegroundColor Yellow
Write-Host ""
Write-Host "   (Por enquanto, execute os comandos acima manualmente)" -ForegroundColor Yellow
Write-Host ""

# 9. Verificar branch atual
$branch = git rev-parse --abbrev-ref HEAD
Write-Host "📂 Branch atual: $branch" -ForegroundColor Cyan
Write-Host ""

# 10. Resumo
Write-Host "✅ Configuração Git concluída!" -ForegroundColor Green
Write-Host ""
Write-Host "📊 Resumo:" -ForegroundColor Cyan
Write-Host "   Email: $GIT_EMAIL" -ForegroundColor White
Write-Host "   Usuário: $GIT_USER" -ForegroundColor White
Write-Host "   Repositório: $REPO_NAME" -ForegroundColor White
Write-Host "   Arquivos: $($addedCount.Count)" -ForegroundColor White
Write-Host ""
Write-Host "🚀 Pronto para fazer upload ao GitHub!" -ForegroundColor Green
Write-Host ""
