# PkgSeal — Architecture Technique

> **Document de référence pour l’architecture du repository, des modules Rust, du desktop Tauri et des frontières de sécurité.**  
> Ce fichier complète l’ADR principal et décrit **comment le projet doit être organisé concrètement**.

- **Projet** : PkgSeal
- **Statut** : Architecture cible initiale
- **Date** : 2026-08-25
- **Plateforme MVP** : Arch Linux
- **Desktop stack** : Tauri 2 + React + TypeScript + Vite
- **UI** : shadcn/ui + Base UI + Tailwind CSS
- **Backend** : Rust
- **Persistence** : SQLite
- **Principe clé** : la structure du repository reflète le produit, pas la technologie

---

# 1. Objectif de cette architecture

PkgSeal doit rester :

- lisible après plusieurs années ;
- modulaire sans devenir un monorepo artificiellement éclaté ;
- robuste face aux changements de sources externes ;
- sécurisé par construction ;
- testable sans modifier la machine de développement ;
- portable vers d’autres distributions ;
- agréable à développer avec Claude Code / Codex sans que les agents mélangent frontend, logique métier et opérations privilégiées.

L’architecture doit éviter deux extrêmes :

1. **monolithe Tauri** où tout finit dans `src-tauri/src/commands.rs` ;
2. **sur-modularisation** avec 30 crates indépendants dès la première semaine.

Les frontières sont créées lorsqu’elles ont une vraie raison :

- responsabilité métier ;
- direction de dépendance ;
- sécurité ;
- testabilité ;
- portabilité.

---

# 2. Vue globale du repository

```text
pkgseal/
│
├── apps/
│   └── desktop/
│       ├── src/
│       ├── src-tauri/
│       ├── public/
│       ├── tests/
│       ├── package.json
│       ├── vite.config.ts
│       └── tsconfig.json
│
├── engine/
│   ├── domain/
│   ├── resolver/
│   ├── policy/
│   └── transactions/
│
├── sources/
│   ├── arch/
│   ├── aur/
│   └── flatpak/
│
├── platform/
│   └── linux/
│
├── testkit/
│
├── fixtures/
│   ├── arch/
│   ├── aur/
│   └── flatpak/
│
├── docs/
│   ├── adr/
│   ├── architecture/
│   ├── security/
│   └── product/
│
├── scripts/
│
├── .github/
│   └── workflows/
│
├── Cargo.toml
├── Cargo.lock
├── package.json
├── bun.lock
├── README.md
├── SECURITY.md
└── LICENSE
```

---

# 3. Règle de dépendance principale

La direction des dépendances doit rester claire :

```text
Desktop UI
    ↓
Tauri application layer
    ↓
Engine
    ↓
Sources / Platform
```

Le domaine ne dépend de rien d’externe.

```text
engine/domain
    ↑
engine/resolver
    ↑
engine/policy
    ↑
engine/transactions
```

Les adapters externes implémentent des contrats définis par le cœur.

```text
sources/arch
sources/aur
sources/flatpak
platform/linux
```

Ils ne définissent pas la logique produit.

---

# 4. Workspace Cargo

Le workspace racine peut commencer avec ces membres :

```toml
[workspace]
resolver = "2"

members = [
    "apps/desktop/src-tauri",
    "engine/domain",
    "engine/resolver",
    "engine/policy",
    "engine/transactions",
    "sources/arch",
    "sources/aur",
    "sources/flatpak",
    "platform/linux",
    "testkit",
]
```

Ne pas créer un crate juste parce qu’un dossier conceptuel existe.

Par exemple, `evidence`, `inspector` et `storage` peuvent initialement vivre dans les modules existants et n’être extraits que lorsqu’une frontière réelle apparaît.

---

# 5. Nommage des packages Cargo

Les dossiers restent courts et sémantiques :

```text
engine/domain/
engine/policy/
sources/aur/
```

Les packages Cargo restent explicites :

```toml
[package]
name = "pkgseal-domain"
```

Exemples :

```text
pkgseal-domain
pkgseal-resolver
pkgseal-policy
pkgseal-transactions
pkgseal-source-arch
pkgseal-source-aur
pkgseal-source-flatpak
pkgseal-platform-linux
pkgseal-testkit
pkgseal-desktop
```

---

# 6. Desktop — organisation complète

Le desktop ne doit pas devenir un frontend React classique rempli de logique métier.

Structure recommandée :

```text
apps/desktop/
│
├── src/
│   ├── app/
│   │   ├── App.tsx
│   │   ├── router.tsx
│   │   ├── providers.tsx
│   │   └── error-boundary.tsx
│   │
│   ├── pages/
│   │   ├── discover/
│   │   ├── application/
│   │   ├── installed/
│   │   ├── transactions/
│   │   ├── history/
│   │   ├── security/
│   │   └── settings/
│   │
│   ├── features/
│   │   ├── search/
│   │   ├── candidate-comparison/
│   │   ├── recommendation/
│   │   ├── evidence/
│   │   ├── aur-review/
│   │   ├── flatpak-permissions/
│   │   ├── transaction-preview/
│   │   └── package-actions/
│   │
│   ├── components/
│   │   ├── ui/
│   │   ├── shell/
│   │   ├── data-display/
│   │   ├── feedback/
│   │   └── navigation/
│   │
│   ├── services/
│   │   ├── ipc/
│   │   ├── queries/
│   │   └── telemetry/
│   │
│   ├── store/
│   │   └── ui-store.ts
│   │
│   ├── lib/
│   │   ├── cn.ts
│   │   ├── format.ts
│   │   ├── errors.ts
│   │   └── keyboard.ts
│   │
│   ├── types/
│   │   ├── api.ts
│   │   └── ui.ts
│   │
│   ├── styles/
│   │   ├── globals.css
│   │   └── tokens.css
│   │
│   ├── assets/
│   └── main.tsx
│
├── src-tauri/
├── public/
├── tests/
└── package.json
```

---

# 7. Desktop — `app/`

`app/` contient uniquement le bootstrap global.

```text
src/app/
├── App.tsx
├── router.tsx
├── providers.tsx
└── error-boundary.tsx
```

Responsabilités :

- router ;
- providers React ;
- theme ;
- query client ;
- error boundaries ;
- application shell.

Interdit :

- appel direct à `pacman` ;
- règles de recommandation ;
- parsing de PKGBUILD ;
- logique de sécurité.

---

# 8. Desktop — `pages/`

Une page représente une destination utilisateur.

```text
pages/discover/
pages/application/
pages/installed/
pages/transactions/
pages/history/
pages/security/
pages/settings/
```

Exemple :

```text
pages/application/
├── ApplicationPage.tsx
├── ApplicationHeader.tsx
├── ApplicationTabs.tsx
└── application-page.test.tsx
```

Une page assemble des features.

Elle ne contient pas les primitives métier.

---

# 9. Desktop — `features/`

Le frontend est organisé par capacités produit.

Exemple :

```text
features/recommendation/
├── RecommendationCard.tsx
├── RecommendationReasons.tsx
├── RecommendationAlternatives.tsx
├── recommendation.types.ts
└── recommendation.test.tsx
```

Autres features :

```text
search
candidate-comparison
evidence
aur-review
flatpak-permissions
transaction-preview
package-actions
```

Une feature peut utiliser :

- composants UI ;
- hooks de query ;
- types IPC.

Elle ne recalcule jamais une recommandation.

---

# 10. Desktop — `components/ui/`

`components/ui/` contient les composants shadcn/Base UI installés localement.

Exemples :

```text
button.tsx
badge.tsx
dialog.tsx
tooltip.tsx
tabs.tsx
input.tsx
scroll-area.tsx
separator.tsx
alert-dialog.tsx
progress.tsx
skeleton.tsx
dropdown-menu.tsx
command.tsx
```

Ces composants sont des primitives.

Ils ne connaissent pas PkgSeal.

Exemple interdit :

```text
components/ui/aur-security-card.tsx
```

Ce composant appartient à une feature.

---

# 11. Desktop — `components/shell/`

Contient la structure de l’application :

```text
AppSidebar.tsx
AppTopbar.tsx
AppContent.tsx
CommandPalette.tsx
WindowControls.tsx
StatusBar.tsx
```

Le shell doit rester indépendant des détails des package sources.

---

# 12. Desktop — `components/data-display/`

Composants visuels réutilisables :

```text
KeyValueRow.tsx
StatusBadge.tsx
SourceBadge.tsx
VersionBadge.tsx
EvidenceList.tsx
FindingList.tsx
EmptyState.tsx
ErrorState.tsx
```

Ces composants peuvent connaître le vocabulaire PkgSeal mais ne déclenchent aucune mutation.

---

# 13. Desktop — `services/ipc/`

Tous les appels Tauri passent par une seule couche typée.

```text
services/ipc/
├── client.ts
├── search.ts
├── applications.ts
├── transactions.ts
├── installed.ts
└── settings.ts
```

Exemple :

```ts
export async function searchApplications(
  input: SearchApplicationsInput,
): Promise<SearchApplicationsResult> {
  return invoke("search_applications", { input });
}
```

Interdit dans les composants :

```ts
invoke("search_applications", ...)
```

L’IPC ne doit pas être dispersé partout dans React.

---

# 14. Desktop — validation IPC

Les réponses IPC sont considérées comme une frontière.

Même si le backend Rust est sous notre contrôle, le frontend valide les payloads importants.

Zod peut être utilisé ici :

```text
services/ipc/schemas/
```

Pas besoin de Zod dans chaque composant.

---

# 15. Desktop — server state

Utiliser TanStack Query pour :

- résultats de recherche ;
- détails d’application ;
- candidats ;
- état installed ;
- evidence ;
- history ;
- transactions.

Ne pas mettre ces données dans Zustand.

---

# 16. Desktop — local UI state

Un store minimal peut être utilisé pour :

- sidebar open/closed ;
- command palette ;
- préférences temporaires ;
- filtres ;
- densité ;
- layout.

Exemple :

```text
store/ui-store.ts
```

Ne pas dupliquer le backend state.

---

# 17. Desktop — routing

Routes proposées :

```text
/
 /discover
 /app/:applicationId
 /installed
 /transactions
 /history
 /security
 /settings
```

Le routing doit refléter les concepts produit, pas les composants.

---

# 18. Desktop — design tokens

Créer les tokens dans :

```text
styles/tokens.css
```

Catégories :

```text
surface
foreground
muted
border
accent
success
warning
danger
info
```

Ne pas mettre des couleurs hex arbitraires dans 80 composants.

---

# 19. Desktop — visual language

PkgSeal doit viser une interface :

- dense ;
- calme ;
- précise ;
- premium ;
- technique sans être austère.

Principes :

```text
1 accent principal
surfaces neutral/zinc
borders subtils
radius modéré
shadow très légère
animations 120-200ms
spacing cohérent
```

Pas d’interface « cyber security » cliché avec du rouge partout.

---

# 20. Desktop — écrans

## Discover

```text
Search
Trending / recent searches
Source availability
Search results
```

## Application Detail

L’écran principal du produit :

```text
Header
Recommendation
Alternatives
Evidence
Permissions
AUR review
Versions
Install preview
```

## Installed

```text
Application
Source
Version
Update availability
Warnings
```

## Transactions

```text
Current
Pending
Completed
Failed
```

## Security

```text
Recent warnings
AUR changes
Broad permissions
Unverified publishers
Policy violations
```

## Settings

```text
Policy
Sources
AUR helper
Flatpak remotes
UI
Advanced
```

---

# 21. Tauri desktop backend

Structure recommandée :

```text
apps/desktop/src-tauri/
│
├── src/
│   ├── main.rs
│   ├── lib.rs
│   │
│   ├── app/
│   │   ├── mod.rs
│   │   ├── state.rs
│   │   └── bootstrap.rs
│   │
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── search.rs
│   │   ├── applications.rs
│   │   ├── installed.rs
│   │   ├── transactions.rs
│   │   └── settings.rs
│   │
│   ├── dto/
│   │   ├── mod.rs
│   │   ├── application.rs
│   │   ├── candidate.rs
│   │   ├── transaction.rs
│   │   └── errors.rs
│   │
│   └── mapping/
│       ├── mod.rs
│       └── domain_to_dto.rs
│
├── capabilities/
├── icons/
├── tauri.conf.json
└── Cargo.toml
```

---

# 22. Tauri — rôle exact

Le crate Tauri sert uniquement de :

```text
composition root
IPC boundary
desktop lifecycle
window management
capability boundary
```

Il ne contient pas :

- policy engine ;
- AUR parser ;
- resolver ;
- transaction engine ;
- SQL métier.

---

# 23. Tauri commands

Les commands doivent être fines.

Exemple :

```rust
#[tauri::command]
async fn search_applications(
    state: State<'_, AppState>,
    input: SearchApplicationsRequest,
) -> Result<SearchApplicationsResponse, ApiError> {
    state.application_service.search(input.into()).await.map(Into::into)
}
```

La command :

- valide ;
- appelle un use case ;
- mappe le résultat ;
- retourne.

Pas de logique métier.

---

# 24. DTO vs Domain

Ne pas exposer directement tous les types Rust du domaine au frontend.

Créer une frontière DTO.

Raisons :

- stabilité IPC ;
- sérialisation contrôlée ;
- découplage UI/domaine ;
- possibilité de masquer des données internes.

---

# 25. `engine/domain`

Le cœur.

Structure :

```text
engine/domain/
└── src/
    ├── lib.rs
    ├── application.rs
    ├── candidate.rs
    ├── source.rs
    ├── publisher.rs
    ├── evidence.rs
    ├── finding.rs
    ├── permission.rs
    ├── recommendation.rs
    ├── transaction.rs
    ├── policy.rs
    ├── ids.rs
    └── error.rs
```

Aucune dépendance :

- Tauri ;
- SQLite ;
- reqwest ;
- pacman ;
- Flatpak.

---

# 26. Newtypes domaine

Exemples :

```rust
pub struct ApplicationId(Uuid);
pub struct CandidateId(Uuid);
pub struct TransactionId(Uuid);
pub struct PackageName(String);
pub struct PublisherName(String);
```

Éviter de passer des `String` partout.

---

# 27. `engine/resolver`

Responsable de l’identité canonique.

```text
engine/resolver/
└── src/
    ├── lib.rs
    ├── normalize.rs
    ├── signals.rs
    ├── matcher.rs
    ├── confidence.rs
    └── ambiguity.rs
```

Entrée :

```text
Vec<PackageCandidate>
```

Sortie :

```text
Vec<ResolvedApplication>
```

---

# 28. Resolver — règle

Le resolver n’effectue aucun accès réseau directement.

Les sources fournissent les métadonnées.

Le resolver travaille sur des données normalisées.

---

# 29. `engine/policy`

Structure :

```text
engine/policy/
└── src/
    ├── lib.rs
    ├── engine.rs
    ├── rules.rs
    ├── presets.rs
    ├── decision.rs
    └── explanation.rs
```

Le policy engine doit être :

- pur ;
- déterministe ;
- testable ;
- sans IO.

---

# 30. `engine/transactions`

Structure :

```text
engine/transactions/
└── src/
    ├── lib.rs
    ├── plan.rs
    ├── operation.rs
    ├── state.rs
    ├── executor.rs
    └── error.rs
```

Ce module produit des plans.

Il ne doit pas avoir un accès root générique.

---

# 31. Sources

Structure :

```text
sources/
├── arch/
├── aur/
└── flatpak/
```

Chaque source possède :

```text
client
parser
mapper
adapter
error
```

Exemple :

```text
sources/aur/src/
├── lib.rs
├── rpc.rs
├── git.rs
├── pkgbuild.rs
├── srcinfo.rs
├── findings.rs
├── adapter.rs
└── error.rs
```

---

# 32. Source Arch

```text
sources/arch/src/
├── lib.rs
├── database.rs
├── package.rs
├── installed.rs
├── adapter.rs
└── error.rs
```

Le choix `libalpm` vs parsing d’outils système doit être décidé dans un ADR séparé.

---

# 33. Source AUR

Le PKGBUILD est hostile par défaut.

L’analyse doit être statique.

Ne jamais :

```text
source PKGBUILD
bash PKGBUILD
eval PKGBUILD
```

Les builds AUR arrivent beaucoup plus tard.

---

# 34. Source Flatpak

```text
sources/flatpak/src/
├── lib.rs
├── remote.rs
├── appstream.rs
├── permissions.rs
├── installed.rs
├── adapter.rs
└── error.rs
```

Doit distinguer :

- source ;
- verification ;
- sandbox ;
- permissions.

---

# 35. `platform/linux`

Tout ce qui est spécifique au système Linux.

```text
platform/linux/
└── src/
    ├── lib.rs
    ├── process.rs
    ├── filesystem.rs
    ├── desktop_entries.rs
    ├── polkit.rs
    ├── privilege.rs
    ├── environment.rs
    └── error.rs
```

Ce crate ne décide jamais quelle variante est recommandée.

---

# 36. Process execution

Créer une abstraction stricte.

Exemple conceptuel :

```rust
pub struct ProcessSpec {
    pub program: KnownBinary,
    pub args: Vec<ValidatedArg>,
    pub timeout: Duration,
}
```

Pas :

```rust
pub fn exec(command: String)
```

---

# 37. Privilèges

Phase finale :

```text
Desktop
  ↓
TransactionPlan
  ↓
PrivilegedRequest
  ↓
Polkit
  ↓
minimal helper
```

Le helper privilégié doit vivre séparément si nécessaire.

Possibilité future :

```text
platform/linux-helper/
```

Ne pas le créer avant d’en avoir besoin.

---

# 38. Storage

Au début, la persistence peut être intégrée dans `apps/desktop/src-tauri` ou dans un module simple.

L’extraire en crate dédié uniquement lorsque l’API devient stable :

```text
engine/storage/
```

Tables conceptuelles :

```text
applications
candidates
evidence
findings
policies
transactions
transaction_events
snapshots
aur_pkgbuild_snapshots
preferences
```

---

# 39. Testkit

```text
testkit/
└── src/
    ├── lib.rs
    ├── builders.rs
    ├── fixtures.rs
    ├── fake_sources.rs
    └── assertions.rs
```

Il permet de construire rapidement :

```rust
candidate()
    .aur()
    .verified_publisher(false)
    .build()
```

pour tester resolver et policy.

---

# 40. Fixtures

Les données réseau réelles utilisées par les tests sont versionnées.

```text
fixtures/aur/brave-bin/
├── rpc.json
├── PKGBUILD
├── .SRCINFO
└── expected.json
```

Les tests unitaires ne doivent pas dépendre du réseau.

---

# 41. Tests frontend

```text
apps/desktop/tests/
├── e2e/
├── fixtures/
└── helpers/
```

Dans `src/`, les tests unitaires restent proches du code.

---

# 42. Tests système

Les opérations réelles package-manager doivent utiliser :

- Arch VM ;
- systemd-nspawn ;
- container adapté lorsque possible.

Jamais le laptop de développement.

---

# 43. Services applicatifs

Au fur et à mesure, le Tauri crate peut composer des services :

```text
SearchService
ApplicationService
RecommendationService
TransactionService
InstalledService
```

Ces services orchestrent :

```text
sources
resolver
policy
storage
```

Ils ne vivent pas dans React.

---

# 44. API interne recommandée

Exemple :

```text
SearchService.search("brave")
ApplicationService.details(application_id)
RecommendationService.evaluate(application_id, policy_id)
TransactionService.plan_install(candidate_id)
TransactionService.execute(transaction_id)
```

Pas :

```text
run_pacman(args)
run_aur_command(args)
```

---

# 45. Error boundaries

Chaque couche possède son type d’erreur.

Exemple :

```text
AurError
ArchSourceError
ResolverError
PolicyError
TransactionError
PlatformError
```

Puis l’application mappe vers :

```text
ApiError
```

L’UI reçoit :

```text
code
message
details
recoverable
```

---

# 46. Configuration

La configuration doit être séparée en :

```text
user preferences
runtime capabilities
source availability
security policy
```

Ne pas avoir un énorme `config.json` fourre-tout.

---

# 47. Source availability

Au démarrage :

```text
Arch       available
AUR        available
Flatpak    available / not installed
```

Une source indisponible ne doit pas empêcher le desktop de démarrer.

---

# 48. Offline mode

Le desktop doit pouvoir afficher :

- installed ;
- cache ;
- historique ;
- anciennes preuves.

Avec un badge :

```text
Cached
Stale
Offline
```

---

# 49. Concurrency

Les sources sont interrogées en parallèle.

```text
search Arch
search AUR
search Flatpak
```

Les résultats sont streamés ou mis à jour progressivement.

Une source lente ne bloque pas les autres.

---

# 50. Security boundaries

Frontière 1 :

```text
WebView → Tauri
```

Frontière 2 :

```text
Tauri → external source
```

Frontière 3 :

```text
Tauri → local package manager
```

Frontière 4 :

```text
unprivileged → privileged helper
```

Chaque frontière valide ses entrées.

---

# 51. Tauri capabilities

Créer des capabilities minimales.

Le frontend ne reçoit pas :

- shell générique ;
- filesystem global ;
- arbitrary process spawning.

Les plugins Tauri sont ajoutés seulement si nécessaires.

---

# 52. Dépendances frontend

Règle :

Ne pas installer une lib pour une fonctionnalité que Base UI / shadcn / navigateur fournit déjà proprement.

Base envisagée :

```text
react
react-dom
@tanstack/react-query
zod
tailwindcss
shadcn/ui components
Base UI
```

Ajouter le router seulement quand les pages le nécessitent réellement.

---

# 53. Dépendances Rust

Limiter les fondations :

```text
serde
serde_json
thiserror
tokio
tracing
uuid
time
```

Puis seulement les dépendances nécessaires aux adapters.

---

# 54. Observability

Utiliser `tracing`.

Chaque transaction possède :

```text
transaction_id
candidate_id
source
operation
duration
result
```

Ne jamais logguer :

- password ;
- token ;
- secret ;
- clipboard.

---

# 55. Git conventions

Branches :

```text
main
dev
feature/...
fix/...
```

Ou trunk-based si le projet reste solo.

Commits petits et cohérents.

Les migrations architecturelles doivent être liées à un ADR.

---

# 56. Documentation

```text
docs/
├── adr/
│   ├── 001-core-architecture.md
│   ├── 002-arch-backend.md
│   └── ...
├── architecture/
│   ├── overview.md
│   ├── desktop.md
│   └── security-boundaries.md
├── security/
│   ├── threat-model.md
│   └── aur.md
└── product/
    ├── principles.md
    └── terminology.md
```

---

# 57. Ce qu’on ne doit pas faire

Éviter :

```text
src-tauri/src/utils.rs
src-tauri/src/helpers.rs
src-tauri/src/common.rs
src-tauri/src/services.rs
```

géants et non structurés.

Éviter aussi :

```text
frontend/src/hooks/useEverything.ts
```

Les noms doivent refléter la responsabilité.

---

# 58. Règle d’extraction d’un nouveau crate

Créer un nouveau crate uniquement si au moins une condition est vraie :

1. frontière de sécurité ;
2. dépendance externe lourde à isoler ;
3. besoin d’être testé indépendamment ;
4. possibilité de réutilisation ;
5. direction de dépendance importante ;
6. module devenu suffisamment autonome.

Sinon garder un module Rust simple.

---

# 59. Architecture MVP réellement recommandée

Ne pas créer toute l’architecture cible le premier jour.

Commencer avec :

```text
pkgseal/
├── apps/desktop/
├── engine/
│   ├── domain/
│   ├── resolver/
│   ├── policy/
│   └── transactions/
├── sources/
│   ├── arch/
│   ├── aur/
│   └── flatpak/
├── platform/linux/
├── testkit/
├── fixtures/
└── docs/
```

C’est suffisant.

---

# 60. Première vertical slice

La première feature complète doit être :

```text
Search "Brave"
   ↓
Arch adapter
AUR adapter
Flatpak adapter
   ↓
Resolver
   ↓
Policy
   ↓
Application Detail UI
```

Aucune installation.

Elle traverse toute l’architecture et valide les frontières.

---

# 61. Definition of Done architecture

L’architecture est considérée saine si :

- React ne connaît aucune commande système ;
- Tauri commands sont fines ;
- domain ne dépend d’aucune source ;
- policy n’effectue aucun IO ;
- resolver est testable avec fixtures ;
- sources sont interchangeables ;
- aucun shell arbitraire n’est exposé ;
- les transactions sont typées ;
- le frontend utilise une couche IPC unique ;
- le design system est centralisé ;
- chaque module possède une responsabilité claire ;
- les tests critiques tournent sans réseau ;
- les opérations privilégiées sont isolables.

---

# 62. Arborescence finale de référence

```text
pkgseal/
│
├── apps/
│   └── desktop/
│       ├── src/
│       │   ├── app/
│       │   ├── pages/
│       │   ├── features/
│       │   ├── components/
│       │   │   ├── ui/
│       │   │   ├── shell/
│       │   │   ├── data-display/
│       │   │   ├── feedback/
│       │   │   └── navigation/
│       │   ├── services/
│       │   │   ├── ipc/
│       │   │   └── queries/
│       │   ├── store/
│       │   ├── lib/
│       │   ├── types/
│       │   ├── styles/
│       │   └── assets/
│       │
│       ├── src-tauri/
│       │   ├── src/
│       │   │   ├── app/
│       │   │   ├── commands/
│       │   │   ├── dto/
│       │   │   └── mapping/
│       │   ├── capabilities/
│       │   └── icons/
│       │
│       ├── tests/
│       ├── public/
│       ├── package.json
│       ├── vite.config.ts
│       └── tsconfig.json
│
├── engine/
│   ├── domain/
│   ├── resolver/
│   ├── policy/
│   └── transactions/
│
├── sources/
│   ├── arch/
│   ├── aur/
│   └── flatpak/
│
├── platform/
│   └── linux/
│
├── testkit/
│
├── fixtures/
│   ├── arch/
│   ├── aur/
│   └── flatpak/
│
├── docs/
│   ├── adr/
│   ├── architecture/
│   ├── security/
│   └── product/
│
├── scripts/
├── .github/
│   └── workflows/
│
├── Cargo.toml
├── Cargo.lock
├── package.json
├── bun.lock
├── README.md
├── SECURITY.md
└── LICENSE
```

---

# 63. Principe final

La structure de PkgSeal doit permettre à quelqu’un d’ouvrir le repository et comprendre immédiatement :

```text
apps       → ce que l’utilisateur exécute
engine     → ce que PkgSeal pense et décide
sources    → d’où viennent les informations
platform   → comment Linux est utilisé
testkit    → comment le cœur est testé
fixtures   → données de test
docs       → pourquoi les décisions existent
```

C’est cette séparation qui doit guider le projet.

> **Le repository doit refléter le produit et ses frontières de confiance, pas simplement les langages utilisés.**
