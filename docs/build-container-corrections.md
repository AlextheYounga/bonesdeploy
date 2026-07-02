Code review bot:

The implementation is correct and matches the plan — one container per deploy via podman create/exec/rm with a Drop guard so failed deploys still clean up, build_image fully excised from config/validation/templates/tests, the Nuxt dist symlink fix and the php→awk swap are in, and all 85 workspace tests pass. The runner-side work is solid. But the templates haven't caught up with the image change, and that will fail your very next test deploy:

1. Nothing bootstraps OS prerequisites on bare debian:bookworm. The old kit default was buildpack-deps:bookworm, which ships curl, ca-certificates, and xz-utils. Bare debian:bookworm has none of them. 01_install_node_deps.sh still goes straight to curl -fsSL ... | tar -xJ — it will die with "curl: command not found" on every Node template. Your own plan doc calls this out under Template Notes ("Build scripts that need OS tools must install them explicitly"), but no template was updated to do it. Each 01_* script needs an apt-get update && apt-get install -y --no-install-recommends curl ca-certificates xz-utils guard up top. This works because scripts run as container-root (rootless podman maps it to the build user on the host) — which is exactly the property that makes your "install anything" model viable.

2. Laravel, Rails, and Django templates are still written for the old host-build era. Laravel's 02_install_php_deps.sh does require_command php and dies — nothing installs PHP or composer now that no image provides them. Rails requires ruby/bundle and pokes at $HOME/.rbenv (a host-machine concept; the container is ephemeral). Django requires python3 (not in bare debian) and tells you to "create a venv on the server" — pure pre-migration thinking. These three need real provisioning scripts, which is the whole point of the model you just built.

3. Django's build script runs manage.py migrate in the build container. Under the new model that's wrong twice: PROJECT.md says migrations belong in prepare scripts (which run as the runtime user with .env and shared paths wired), and the build container deliberately has no .env or database access, so it can't work anyway.

Questions you're not asking:

- What happens to containers when bonesremote dies uncleanly? Drop handles panics and errors, but SIGKILL or a host reboot mid-deploy leaves a sleep infinity container running forever, and your unique-per-deploy names mean they accumulate silently. Consider a deterministic name per site (bonesdeploy-build-<site>) with podman rm -f before create — that self-heals orphans and acts as a concurrent-deploy guard — or label containers and prune stale ones at build start.
- Who updates the base image? debian:bookworm is a floating tag and --pull=missing freezes whatever was pulled first, forever, per host. You've traded per-framework image maintenance for one base image nobody refreshes. Decide the story: pin a digest and bump deliberately, or podman pull during provisioning/doctor.
- What actually belongs in a sealed release? Promote still copies the entire mutated context — including node_modules, .nuxt, and (pre-existing, but worth fixing now) the <script>.log files that run_scripts.rs:51 writes into the build context itself. Your releases will contain their own build logs and ~1200 packages of dead weight, ×5 retained releases. The "prune to deployable shape" step still doesn't exist in any template.
- Should the build container have resource limits? It has no --memory or --pids-limit, so one npm ci gone wrong can OOM a shared server hosting other sites. Cheap insurance given the multi-tenant design.
- Egress: the container has unrestricted network — inherent to "download anything," but it means a malicious dependency can exfiltrate your source. Worth a sentence in the security notes so it's a documented ceiling rather than an oversight.

One housekeeping item: deleting the build_image validation test left an unused validate_site_dataset import warning in crates/bonesremote/src/commands/site.rs.

The Rust side is done; the template overhaul (items 1–3) is the real remaining work, and it's what stands between you and a green test deploy on makebabies.


My responses:
1. Ah you're right. It might not be enough that bonesinfra has installed all these packages to the host machine, they may not be available in the container. I have considered this problem before, and my solution was to create a 00_* file that is run
  automatically before every build and handles these normal cross-cutting base concerns. This will be a hardcoded file that is not exposed to the user, but is simply stored in the project (still as an embedded bash file though) and is run once the container is
  started.
2. Noted. Good catch.
3. Yes, this should be in the prepare scripts. Good catch. 

Further answers/questions:
1. Yes, let's create a check to ensure there are never any orphaned builds running by deterministically checking for existing running containers. 
2. This will be future problems, but good to note. 
3. Should we consider running a git clone command to get the site into the container? 
4. We will handle this in a later correction, but good to note. 
5. We will constrain this later, but we will keep this fully open for now. 