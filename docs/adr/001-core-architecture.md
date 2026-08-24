# ADR-001 — Architecture de PkgSeal

- **Projet** : PkgSeal
- **Type** : Application desktop Linux de découverte, comparaison, évaluation et installation de logiciels
- **Statut** : Accepted
- **Date** : 2026-08-25
- **Décision** : Architecture initiale pour le MVP et la montée en production
- **Portée initiale** : Arch Linux
- **Extension future** : autres distributions Linux et autres sources de paquets

---

## 1. Contexte

Installer une application sous Linux, et particulièrement sous Arch Linux, demande souvent de choisir entre plusieurs sources :

- dépôts officiels Arch ;
- AUR ;
- Flatpak / Flathub ;
- paquets fournis directement par l’éditeur ;
- AppImage ;
- Snap ;
- installateurs ou scripts tiers.

Le problème n’est pas seulement de trouver un paquet. Le vrai problème est de déterminer :

1. quelles variantes correspondent réellement à la même application ;
2. laquelle est la plus pertinente pour la machine et la politique utilisateur ;
3. quelle est la provenance du paquet ;
4. quelles garanties de sécurité existent ;
5. quels compromis sont introduits par chaque format ;
6. ce qui sera réellement exécuté avant l’installation.

Les gestionnaires existants répondent généralement à une seule partie du problème :

- `pacman` gère très bien les dépôts Arch ;
- les helpers AUR facilitent l’accès à l’AUR ;
- Flatpak gère les applications Flatpak ;
- les stores graphiques améliorent la découverte ;
- certains outils agrègent plusieurs sources.

PkgSeal doit se placer au-dessus de ces mécanismes existants et agir comme une **couche de résolution, de provenance, d’explication et de politique**, sans réimplémenter les gestionnaires de paquets eux-mêmes.

---

## 2. Vision produit

PkgSeal doit permettre à l’utilisateur d’exprimer une intention simple :

> « Je veux installer cette application. »

Puis transformer cette intention en une décision explicable :

> « Voici les variantes disponibles, leurs différences, leurs garanties, leurs risques et celle que PkgSeal recommande selon ta politique. »

Le pipeline conceptuel est :

```text
Search
  ↓
Discover candidates
  ↓
Resolve identities
  ↓
Collect evidence
  ↓
Apply policy
  ↓
Recommend
  ↓
Preview transaction
  ↓
Install / Remove
```

PkgSeal n’est donc pas un simple « App Store pour Arch ».

Il s’agit d’un **package provenance & decision layer**.

---

## 3. Objectifs

### 3.1 Objectifs MVP

Le MVP doit :

- rechercher une application dans plusieurs sources ;
- identifier les résultats représentant la même application ;
- afficher toutes les variantes disponibles ;
- collecter et afficher des preuves de provenance ;
- analyser les permissions et caractéristiques pertinentes ;
- analyser statiquement les fichiers AUR importants ;
- comparer les variantes ;
- appliquer une politique configurable ;
- recommander une variante en expliquant pourquoi ;
- prévisualiser précisément une transaction ;
- installer et supprimer proprement une application ;
- conserver l’historique des décisions et des transactions ;
- rester rapide, lisible et visuellement premium.

### 3.2 Objectifs qualité

PkgSeal doit être :

- déterministe dans ses décisions de sécurité ;
- testable sans machine utilisateur réelle ;
- robuste face aux changements d’API ;
- sécurisé par défaut ;
- modulaire ;
- maintenable ;
- observable ;
- accessible au clavier ;
- cohérent avec les bonnes pratiques Linux.

---

## 4. Non-objectifs initiaux

Le MVP ne doit pas :

- remplacer `pacman` ;
- remplacer Flatpak ;
- implémenter son propre système de build AUR ;
- exécuter arbitrairement des commandes root ;
- agir comme antivirus ;
- promettre qu’un paquet est « sûr » ;
- inventer un score de sécurité opaque ;
- gérer les drivers ou le kernel comme un store grand public ;
- supporter toutes les distributions dès la v0.1 ;
- lancer automatiquement des upgrades système complets ;
- utiliser un LLM pour prendre les décisions critiques ;
- effectuer une installation silencieuse sans preview.

---

## 5. Principes d’architecture

### 5.1 Le frontend n’est pas une frontière de confiance

Le WebView doit être considéré comme non privilégié.

Il ne doit jamais pouvoir fournir directement :

- une commande shell ;
- un chemin root arbitraire ;
- des arguments arbitraires à `pacman` ;
- une unité systemd ;
- un script à exécuter ;
- un mot de passe.

Le frontend demande des **intentions typées**.

Exemple acceptable :

```text
InstallCandidate(candidate_id)
```

Exemple interdit :

```text
RunAsRoot("pacman -S ...")
```

### 5.2 Les décisions de sécurité sont explicables

Une recommandation est construite à partir :

```text
Evidence
  ↓
Policy
  ↓
Recommendation
```

L’UI ne contient aucune logique cachée décidant quelle source est « meilleure ».

### 5.3 Les outils Linux existants restent autoritatifs

PkgSeal orchestre les mécanismes natifs.

Il ne réimplémente pas leurs fonctionnalités sensibles.

### 5.4 Les données externes sont non fiables par défaut

Sont traités comme contenus potentiellement hostiles :

- PKGBUILD ;
- `.install` ;
- métadonnées AUR ;
- manifestes Flatpak ;
- descriptions ;
- URLs ;
- icônes distantes ;
- fichiers téléchargés ;
- données d’API.

---

## 6. Stack technique

## 6.1 Desktop

**Tauri 2**

Raisons :

- core Rust ;
- faible empreinte ;
- bonne séparation frontend/backend ;
- capabilities configurables ;
- intégration native desktop ;
- distribution Linux adaptée ;
- pas besoin d’embarquer un runtime Electron complet.

## 6.2 Frontend

- React
- TypeScript `strict`
- Vite
- Tailwind CSS
- shadcn/ui
- Base UI
- style de base compact proche de Rhea
- TanStack Query pour les données asynchrones
- TanStack Router si un vrai routing devient nécessaire
- Zod uniquement aux frontières frontend où il apporte une validation utile

Le frontend ne doit contenir aucune règle métier de sécurité.

## 6.3 Backend

Rust stable.

Principes :

- pas de `unwrap()` dans le code de production ;
- erreurs typées ;
- `thiserror` pour les bibliothèques ;
- séparation stricte IO / domaine ;
- interfaces explicites ;
- aucun shell implicite.

## 6.4 Persistance

SQLite local.

Usage :

- cache ;
- historique ;
- préférences ;
- règles de policy ;
- snapshots de métadonnées ;
- hashes de PKGBUILD ;
- décisions antérieures ;
- transactions.

Aucun secret inutile n’est stocké.

---

## 7. Architecture logique

```text
┌─────────────────────────────────────────────────────────┐
│                        UI                               │
│ React + shadcn/ui + Base UI                            │
└───────────────────────┬─────────────────────────────────┘
                        │ typed Tauri commands
                        ▼
┌─────────────────────────────────────────────────────────┐
│                  Application Layer                      │
│ Use cases / orchestration                              │
└───────────┬────────────┬────────────┬───────────────────┘
            │            │            │
            ▼            ▼            ▼
      ┌──────────┐  ┌──────────┐  ┌────────────┐
      │ Resolver │  │  Policy  │  │ Inspector  │
      └────┬─────┘  └────┬─────┘  └─────┬──────┘
           │             │              │
           └─────────────┴───────┬──────┘
                                 ▼
                         ┌──────────────┐
                         │   Domain     │
                         └──────┬───────┘
                                │
               ┌────────────────┼────────────────┐
               ▼                ▼                ▼
        ┌────────────┐    ┌────────────┐   ┌────────────┐
        │ Arch source │    │ AUR source │   │ Flatpak    │
        └────────────┘    └────────────┘   └────────────┘
                                │
                                ▼
                     ┌──────────────────┐
                     │ Transaction Core │
                     └────────┬─────────┘
                              ▼
                     ┌──────────────────┐
                     │ Privileged helper │
                     │ + polkit          │
                     └──────────────────┘
```

---

## 8. Workspace Rust

Structure proposée :

```text
pkgseal/
├── apps/
│   └── desktop/
│       ├── src/
│       ├── src-tauri/
│       └── package.json
│
├── crates/
│   ├── pkgseal-domain/
│   ├── pkgseal-app/
│   ├── pkgseal-source-arch/
│   ├── pkgseal-source-aur/
│   ├── pkgseal-source-flatpak/
│   ├── pkgseal-resolver/
│   ├── pkgseal-policy/
│   ├── pkgseal-inspector/
│   ├── pkgseal-transactions/
│   ├── pkgseal-storage/
│   ├── pkgseal-platform-linux/
│   └── pkgseal-testkit/
│
├── fixtures/
│   ├── arch/
│   ├── aur/
│   └── flatpak/
│
├── docs/
│   ├── adr/
│   ├── security/
│   └── architecture/
│
├── scripts/
├── Cargo.toml
└── README.md
```

---

## 9. Domaine

Le domaine doit rester indépendant de Tauri, SQLite et des APIs externes.

### 9.1 Entités principales

```text
ApplicationIdentity
PackageCandidate
PackageSource
PackageVersion
Publisher
Evidence
RiskFinding
Permission
Recommendation
Policy
TransactionPlan
TransactionResult
```

### 9.2 PackageSource

Exemple conceptuel :

```rust
pub enum PackageSource {
    ArchOfficial,
    Aur,
    Flatpak,
}
```

Les futures variantes peuvent inclure :

```text
AppImage
Snap
VendorRepository
Nix
Debian
Fedora
```

sans modifier le cœur du produit.

### 9.3 PackageCandidate

Un candidat représente une variante installable.

Il contient notamment :

- identifiant interne ;
- identité d’application résolue ;
- source ;
- package ID ;
- version ;
- architecture ;
- publisher déclaré ;
- homepage ;
- source repository ;
- méthode d’installation ;
- preuves ;
- findings ;
- permissions ;
- disponibilité locale ;
- état installé ;
- métadonnées brutes conservées séparément si nécessaire.

---

## 10. Source adapters

Chaque source implémente une interface commune.

Conceptuellement :

```rust
trait PackageSourceAdapter {
    async fn search(&self, query: &SearchQuery) -> Result<Vec<PackageCandidate>>;
    async fn inspect(&self, candidate: &CandidateRef) -> Result<SourceEvidence>;
    async fn installed(&self) -> Result<Vec<InstalledPackage>>;
}
```

Les opérations d’installation ne doivent pas forcément appartenir au même trait.

La lecture et la mutation restent séparées.

---

## 11. Adapter Arch

### 11.1 Lecture

La source Arch doit pouvoir fournir :

- nom ;
- version ;
- repo ;
- description ;
- architecture ;
- maintainer disponible ;
- signature / état pertinent ;
- dépendances ;
- URL ;
- taille ;
- statut installé.

### 11.2 Installation

L’installation passe par une transaction typée.

PkgSeal ne construit jamais :

```text
sh -c "sudo pacman ..."
```

Les arguments sont construits séparément par le backend.

---

## 12. Adapter AUR

L’AUR est une source communautaire et doit être traité comme telle.

### 12.1 Données analysées

Pour chaque candidat AUR :

- métadonnées RPC ;
- PKGBUILD ;
- `.SRCINFO` ;
- fichiers `.install` ;
- patches pertinents ;
- URLs des sources ;
- checksums ;
- historique de diff si disponible ;
- maintainer ;
- popularité uniquement comme contexte, jamais comme preuve de sécurité.

### 12.2 Règle critique

**PkgSeal ne source jamais un PKGBUILD.**

Le PKGBUILD est analysé comme texte non fiable.

### 12.3 Findings statiques

PkgSeal doit pouvoir signaler notamment :

- `curl | sh` ;
- `wget | sh` ;
- `eval` ;
- `sudo` ;
- writes root explicites ;
- `chmod +s` ;
- `chown root` ;
- décodage base64 suspect ;
- exécution de contenu téléchargé ;
- network fetch inattendu dans les phases de build ;
- scripts install ;
- hooks ;
- commandes obfusquées.

Un finding n’équivaut pas à « malware ».

Il doit être décrit factuellement.

### 12.4 Diff d’update

Lors d’une mise à jour AUR, PkgSeal doit comparer le précédent snapshot au nouveau.

Exemple :

```text
PKGBUILD changed

Version:
1.2.0 → 1.3.0

Source URL:
changed

Checksum:
changed

Build logic:
unchanged
```

C’est une fonctionnalité centrale.

---

## 13. Adapter Flatpak

Pour Flatpak / Flathub, PkgSeal doit exposer :

- application ID ;
- remote ;
- verification publisher si disponible ;
- permissions ;
- filesystem access ;
- sockets ;
- devices ;
- D-Bus access ;
- Wayland/X11 ;
- network ;
- portals ;
- runtime ;
- taille ;
- statut installé.

L’UI doit distinguer clairement :

```text
Verified publisher
```

de :

```text
Secure application
```

La vérification de l’identité ne garantit pas l’absence de vulnérabilité.

---

## 14. Resolver

Le resolver fusionne les candidats représentant la même application.

Exemple :

```text
brave-bin
com.brave.Browser
Brave Browser
```

doivent pouvoir devenir :

```text
ApplicationIdentity {
    canonical_name: "Brave Browser"
}
```

### 14.1 Signaux déterministes

Ordre de priorité initial :

1. application ID connu ;
2. reverse-domain ID ;
3. homepage ;
4. source repository ;
5. publisher ;
6. package metadata ;
7. desktop file identifiers ;
8. binary name ;
9. normalized product name ;
10. fuzzy matching contrôlé.

### 14.2 IA

Aucun LLM n’est utilisé pour décider de l’identité dans le MVP.

Un modèle pourra plus tard proposer un rapprochement ambigu, mais :

- il ne fusionnera jamais automatiquement un résultat de faible confiance ;
- la décision finale restera explicable ;
- aucune recommandation sécurité ne dépendra directement d’un LLM.

---

## 15. Evidence model

PkgSeal ne produit pas un « score sécurité 92/100 ».

Chaque conclusion doit être liée à des preuves.

Exemples :

```text
Publisher-supported install method
Package from Arch official repositories
Flatpak publisher verified
Checksum available
Checksum validated
Signed package
Community-maintained AUR recipe
Broad filesystem permission
Host D-Bus access
Build script changed
Vendor documentation references this package
```

Chaque evidence contient :

- type ;
- valeur ;
- provenance ;
- date de collecte ;
- niveau de confiance ;
- référence ;
- éventuelle expiration.

---

## 16. Policy engine

Le moteur de policy est pur et déterministe.

Entrée :

```text
Candidates + Evidence + UserPolicy
```

Sortie :

```text
Recommendation + Reasons + Warnings
```

### 16.1 Policies initiales

#### Balanced

Optimise :

- provenance ;
- support éditeur ;
- sécurité ;
- intégration système ;
- maintenance.

#### Native First

Favorise les paquets natifs lorsque les garanties sont comparables.

#### Sandbox First

Favorise les variantes sandboxées lorsque leurs permissions restent raisonnables.

#### Maximum Review

N’automatise aucune source communautaire sans validation approfondie.

### 16.2 Règle importante

L’ordre :

```text
Arch > Flatpak > AUR
```

ne doit jamais être codé en dur comme vérité universelle.

Une recommandation dépend :

- du logiciel ;
- des recommandations éditeur ;
- de la provenance ;
- des permissions ;
- des caractéristiques techniques ;
- de la policy choisie.

---

## 17. Recommendation

Exemple :

```text
Recommended

brave-bin · AUR

Why PkgSeal recommends this:
✓ documented installation method
✓ native Chromium sandbox behavior
✓ checksum present
⚠ community-maintained AUR recipe
```

Une recommandation doit exposer :

- candidat recommandé ;
- raisons positives ;
- compromis ;
- warnings ;
- alternatives ;
- niveau de confiance ;
- données manquantes.

---

## 18. Transaction engine

Aucune installation n’est déclenchée directement depuis une source adapter.

Toutes les mutations passent par :

```text
TransactionPlan
```

### 18.1 Exemple

```text
InstallTransaction
├── source: Arch
├── package: foo
├── expected_download: ...
├── expected_disk_change: ...
├── privileges_required: true
└── operations:
    └── InstallPackage(foo)
```

### 18.2 États

```text
Planned
AwaitingConfirmation
Authorizing
Running
Succeeded
Failed
Cancelled
```

### 18.3 Propriétés

Une transaction doit être :

- sérialisable ;
- journalisable ;
- inspectable avant exécution ;
- reproductible autant que possible ;
- liée à l’application et au candidat source.

---

## 19. Élévation de privilèges

PkgSeal ne stocke jamais le mot de passe sudo.

Les opérations privilégiées doivent évoluer vers :

```text
Desktop app
  ↓
typed privileged request
  ↓
Polkit authorization
  ↓
small privileged helper
  ↓
specific package operation
```

Le helper privilégié doit être minimal.

API acceptable :

```text
install_arch_packages(Vec<PackageName>)
remove_arch_packages(Vec<PackageName>)
```

API interdite :

```text
run_command_as_root(String)
```

Le helper ne doit pas devenir un shell root distant contrôlé par le WebView.

---

## 20. Shell execution policy

Interdit par défaut :

```rust
Command::new("sh")
    .arg("-c")
    .arg(dynamic_input)
```

Toute exécution utilise :

- binaire connu ;
- arguments distincts ;
- allowlist ;
- validation stricte ;
- environnement explicitement contrôlé si nécessaire ;
- timeouts ;
- capture stdout/stderr ;
- limites de taille de logs.

---

## 21. Design system

L’application doit être visuellement premium sans sacrifier la densité d’information.

### 21.1 Base

- shadcn/ui ;
- Base UI ;
- Tailwind CSS ;
- primitives accessibles ;
- composants copiés et maîtrisés dans le repository.

### 21.2 Direction visuelle

- surfaces neutres ;
- dark mode prioritaire mais light mode supporté ;
- une couleur d’accent ;
- contrastes propres ;
- borders fines ;
- radius modéré ;
- ombres discrètes ;
- animations 120–200 ms ;
- typographie nette ;
- densité desktop ;
- aucune surcharge décorative.

### 21.3 Écrans MVP

```text
Discover
Search Results
Application Detail
Installed
Transaction
History
Security / Findings
Settings / Policy
```

### 21.4 Fiche application

C’est l’écran le plus important.

Sections :

```text
Overview
Sources
Why recommended
Security evidence
Permissions
PKGBUILD review
Versions
Transaction preview
```

---

## 22. UX de sécurité

La sécurité ne doit pas devenir une collection d’icônes rouges.

Les informations sont classées en :

```text
Evidence
Warning
Review required
Block
```

PkgSeal ne doit bloquer une opération que pour une règle forte et explicitement définie.

Un utilisateur avancé peut outrepasser certains warnings, avec confirmation claire.

---

## 23. Installed state

PkgSeal doit construire un inventaire local :

```text
InstalledApplication
├── canonical application
├── source
├── package ID
├── version
├── install date if known
├── update state
└── previous evidence snapshot
```

Cela permet :

- comparaison d’updates ;
- détection de changement de source ;
- historique ;
- future migration entre formats.

---

## 24. Persistence model

Tables conceptuelles :

```text
applications
package_candidates
candidate_sources
evidence
findings
policies
policy_rules
transactions
transaction_events
installed_packages
snapshots
aur_pkgbuild_snapshots
user_preferences
```

Les caches distants ont un TTL.

Les données importantes de provenance sont timestampées.

---

## 25. Network layer

Toutes les requêtes HTTP backend passent par une couche commune.

Exigences :

- HTTPS ;
- timeout ;
- retry limité ;
- backoff ;
- user-agent PkgSeal ;
- taille maximale ;
- parsing défensif ;
- cache ;
- validation des URLs ;
- aucune redirection aveugle vers des schémas non autorisés.

---

## 26. Cache

PkgSeal doit fonctionner correctement même si une source distante est lente.

Niveaux :

```text
memory cache
SQLite cache
network
```

L’UI doit indiquer si une information est :

```text
Live
Cached
Stale
Unavailable
```

---

## 27. Logging

Rust utilise un logging structuré.

Les logs ne doivent jamais contenir :

- passwords ;
- tokens ;
- secrets ;
- contenu clipboard ;
- données utilisateur sans besoin.

Les transactions disposent d’un journal dédié.

---

## 28. Error model

Les erreurs utilisateur doivent être compréhensibles.

Ne jamais afficher uniquement :

```text
Exit code 1
```

À la place :

```text
Installation failed

pacman could not resolve dependency X.

Details
...
```

Le détail brut reste accessible.

---

## 29. Tests

La stratégie de test est une exigence d’architecture.

### 29.1 Unit tests

Priorité très élevée pour :

- resolver ;
- policy ;
- normalization ;
- evidence ;
- findings ;
- transaction planning.

### 29.2 Fixtures

Aucune suite principale ne dépend du réseau réel.

```text
fixtures/
├── arch/
├── aur/
└── flatpak/
```

Les réponses externes sont snapshotées et versionnées.

### 29.3 Integration tests

Testent :

- adapters ;
- storage ;
- parsing ;
- transaction planner ;
- Tauri command boundaries.

### 29.4 System tests

Les vraies opérations package manager utilisent un environnement jetable.

Options :

- Arch VM ;
- systemd-nspawn ;
- image CI dédiée.

Jamais la machine de développement principale.

### 29.5 Frontend

- Vitest ;
- Testing Library ;
- tests clavier ;
- états loading/error ;
- transaction confirmation ;
- navigation.

### 29.6 E2E

Scénarios critiques :

```text
Search → Resolve → Inspect → Recommend
Search → Candidate → Transaction Preview
Installed → Remove Preview
AUR update → PKGBUILD Diff
Permission warning → User confirmation
```

---

## 30. CI

Chaque pull request doit exécuter :

```text
Frontend
├── lint
├── typecheck
├── unit tests
└── build

Rust
├── cargo fmt --check
├── cargo clippy -- -D warnings
├── cargo test
└── cargo build

Security
├── dependency audit
├── secret scanning
└── forbidden-pattern checks
```

Aucun merge si la CI est rouge.

---

## 31. Dependency policy

Toute nouvelle dépendance doit répondre à une justification.

Critères :

- maintenance ;
- surface d’attaque ;
- licence ;
- taille ;
- maturité ;
- nécessité réelle.

Éviter les dépendances pour des fonctions triviales.

Le lockfile est versionné.

---

## 32. Supply-chain du projet

PkgSeal doit appliquer à lui-même une politique stricte :

- dépendances lockées ;
- Dependabot/Renovate ou équivalent ;
- audits réguliers ;
- releases reproductibles autant que possible ;
- artifacts signés à terme ;
- SBOM à terme ;
- provenance des releases à terme.

---

## 33. Releases

Phases :

```text
0.1-alpha
read-only package intelligence

0.2-alpha
recommendations + evidence

0.3-alpha
safe installation transactions

0.4-beta
AUR review + update diff

0.5-beta
polish + reliability

1.0
stable Arch release
```

---

## 34. Roadmap de développement

## Phase 0 — Foundation

Livrables :

- monorepo ;
- Tauri ;
- React/Vite ;
- shadcn/ui + Base UI ;
- Rust workspace ;
- CI ;
- lint ;
- test harness ;
- SQLite ;
- ADRs ;
- threat model initial.

Gate :

```text
clean build
zero clippy warning
frontend strict typecheck
CI green
```

## Phase 1 — Read-only Package Explorer

Sources :

- Arch ;
- AUR ;
- Flatpak.

Fonctions :

- search ;
- details ;
- installed state ;
- source badges ;
- version comparison.

Aucune mutation système.

## Phase 2 — Identity Resolver

- normalization ;
- app identities ;
- candidate grouping ;
- confidence ;
- ambiguity handling.

## Phase 3 — Evidence & Inspector

- provenance ;
- verification ;
- Flatpak permissions ;
- AUR PKGBUILD parser ;
- findings ;
- snapshots.

## Phase 4 — Policy Engine

- policies ;
- recommendations ;
- reasons ;
- alternatives ;
- unit test matrix.

## Phase 5 — Transaction Preview

Toujours sans exécution :

- install plans ;
- remove plans ;
- disk/download estimates ;
- privilege indicator ;
- confirmation UX.

## Phase 6 — Arch Transactions

- polkit/helper ;
- pacman integration ;
- progress ;
- logs ;
- failures.

## Phase 7 — Flatpak Transactions

- install/remove ;
- permissions ;
- portal-related warnings.

## Phase 8 — AUR Transactions

Uniquement après stabilisation du modèle de sécurité.

- helper integration ;
- PKGBUILD review ;
- diff ;
- confirmation ;
- build lifecycle.

## Phase 9 — Product Polish

- command palette ;
- keyboard-first UX ;
- animations ;
- HiDPI ;
- accessibility ;
- visual QA ;
- performance.

---

## 35. Premier milestone produit

Le premier milestone ne doit rien installer.

Critère de réussite :

Une recherche pour :

```text
Brave
Bitwarden
Discord
Spotify
VS Code
Steam
Obsidian
```

doit :

1. trouver les candidats disponibles ;
2. les regrouper correctement ;
3. afficher les différences ;
4. collecter les preuves ;
5. appliquer une policy ;
6. expliquer la recommandation.

Si ce pipeline n’est pas fiable, l’installation ne doit pas être implémentée.

---

## 36. Threat model initial

Acteurs / risques :

### Source externe malveillante

Peut fournir :

- métadonnées piégées ;
- PKGBUILD dangereux ;
- URLs hostiles ;
- texte trompeur.

Mitigation :

- aucune exécution durant l’analyse ;
- parsers défensifs ;
- output encoding ;
- network restrictions ;
- findings.

### WebView compromis

Risque :

- tentative d’exécution privilégiée.

Mitigation :

- capabilities minimales ;
- API Tauri typée ;
- aucune commande shell générique ;
- privileged helper fermé.

### Package communautaire hostile

Risque :

- exécution durant build/install.

Mitigation :

- review ;
- diff ;
- transaction preview ;
- environnement de build contrôlé ;
- warnings ;
- aucune confiance automatique.

### Confused deputy

Risque :

- le frontend utilise le backend privilégié pour effectuer une opération non prévue.

Mitigation :

- opérations fortement typées ;
- allowlists ;
- validation backend ;
- revalidation dans le helper privilégié.

---

## 37. Règles de coding

### Rust

- `cargo fmt` obligatoire ;
- `cargo clippy -D warnings` ;
- pas de `unwrap` production ;
- erreurs explicites ;
- structs immutables par défaut ;
- enums pour états fermés ;
- pas de `String` quand un newtype métier apporte de la sécurité ;
- IO aux frontières.

### TypeScript

- `strict: true` ;
- pas de `any` hors boundary exceptionnelle ;
- pas de logique policy dans React ;
- composants UI découplés du transport Tauri ;
- validation des réponses IPC.

---

## 38. Newtypes recommandés

Éviter :

```rust
fn install(name: String, source: String)
```

Préférer :

```text
PackageName
CandidateId
ApplicationId
SourceId
TransactionId
PublisherId
```

Cela réduit les erreurs de mélange de paramètres.

---

## 39. Performance

Objectifs initiaux :

- démarrage perçu rapide ;
- recherche progressive ;
- résultats Arch/AUR/Flatpak affichés au fur et à mesure ;
- aucune UI bloquée par une source lente ;
- cache persistant ;
- virtualization pour longues listes ;
- lazy loading des analyses coûteuses.

---

## 40. Accessibilité

Obligatoire dès le design system :

- navigation clavier ;
- focus visible ;
- labels accessibles ;
- contrastes ;
- reduced motion ;
- pas d’informations transmises uniquement par couleur.

---

## 41. Branding technique

Le terme **Seal** peut devenir un concept UX sans devenir un score opaque.

Exemples :

```text
PkgSeal Verified Evidence
PkgSeal Review Required
PkgSeal Warning
```

À éviter :

```text
100% Safe
Secure Seal
Guaranteed Safe
```

PkgSeal informe et recommande ; il ne garantit jamais qu’un logiciel est exempt de vulnérabilité ou de comportement malveillant.

---

## 42. Évolution multi-distro

Le cœur doit être conçu pour ne pas dépendre d’Arch.

À terme :

```text
pkgseal-source-debian
pkgseal-source-fedora
pkgseal-source-nix
pkgseal-source-snap
pkgseal-source-appimage
```

Le domaine, resolver et policy engine restent inchangés.

---

## 43. Décisions retenues

### D1 — Tauri 2

**Accepted**

Pour séparer UI Web et core Rust avec une empreinte adaptée au desktop Linux.

### D2 — React + Vite, pas Next.js

**Accepted**

Aucun besoin SSR/RSC. Réduction de complexité.

### D3 — shadcn/ui + Base UI

**Accepted**

Base de composants accessible, moderne, contrôlable et adaptée à une UI desktop premium.

### D4 — Rust workspace modulaire

**Accepted**

Permet une isolation forte entre domaine, adapters, policy, sécurité et transactions.

### D5 — SQLite local

**Accepted**

Suffisant pour cache, historique et snapshots sans backend distant.

### D6 — Policy déterministe

**Accepted**

Aucun LLM dans la décision de sécurité critique.

### D7 — Evidence over score

**Accepted**

Pas de score de sécurité opaque.

### D8 — Typed privileged operations

**Accepted**

Aucune commande root arbitraire.

### D9 — Read-only first

**Accepted**

Les mutations système arrivent après validation du resolver et du moteur de recommandation.

### D10 — Arch first, distro-agnostic core

**Accepted**

Le MVP cible Arch sans enfermer l’architecture dans Arch.

---

## 44. Questions ouvertes

À résoudre dans des ADR séparés :

1. `libalpm` direct vs orchestration `pacman` pour certaines lectures/opérations ;
2. helper AUR retenu (`paru`, `yay`, build pipeline interne contrôlé) ;
3. format exact du moteur de policy ;
4. stratégie de signature des releases ;
5. stratégie de sandbox des builds AUR ;
6. fournisseur de métadonnées éditeur ;
7. méthode de vérification de recommandation officielle d’un éditeur ;
8. stratégie de migration d’une application d’une source à une autre ;
9. mode offline ;
10. identité canonique cross-distro.

---

## 45. Definition of Done v0.1-alpha

La v0.1-alpha est considérée terminée lorsque :

- l’application démarre proprement sur Arch/Hyprland ;
- le design system est stable ;
- la recherche Arch/AUR/Flatpak fonctionne ;
- les résultats sont regroupés correctement pour le corpus de référence ;
- l’UI montre provenance et variantes ;
- le moteur d’evidence fonctionne ;
- le policy engine produit une recommandation explicable ;
- aucune mutation système n’est possible ;
- les tests critiques sont verts ;
- la CI est verte ;
- les fixtures permettent les tests offline ;
- aucun secret n’est stocké ;
- aucun shell arbitraire n’est exposé.

---

## 46. Résumé exécutif

PkgSeal sera construit comme un **moteur local de résolution et de provenance de logiciels Linux**, présenté dans une application desktop moderne.

Le produit ne remplace pas les package managers.

Il :

```text
trouve
→ regroupe
→ inspecte
→ explique
→ recommande
→ prévisualise
→ orchestre
```

L’architecture privilégie :

- Rust pour le domaine et les opérations sensibles ;
- Tauri pour la frontière desktop ;
- React + shadcn/Base UI pour une expérience visuelle premium ;
- adapters indépendants par source ;
- recommendations déterministes ;
- preuves plutôt que scores ;
- transactions typées ;
- Polkit pour les privilèges ;
- tests avant mutations ;
- Arch comme première plateforme sans verrouiller le produit à Arch.

La règle centrale du projet est :

> **PkgSeal must never hide a trust decision behind an Install button.**

Chaque installation doit être compréhensible, explicable et inspectable avant de modifier la machine.
