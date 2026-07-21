**Minimum Sane Fix Plan**

1. **Add one setup hardening module**
   - Create `src/bonesinfra/deploys/setup/kernel_hardening.py`.
   - It should do only two things:
     - render `/etc/modprobe.d/disable-algif.conf`
     - run `rmmod algif_aead 2>/dev/null || true`
   - No broad sysctl hardening, no seccomp, no extra kernel module blacklist framework.

2. **Add one tiny asset**
   - Create `src/bonesinfra/assets/modprobe/disable-algif.conf.j2`.
   - Contents:
     ```conf
     install algif_aead /bin/false
     ```
   - Keep it specific to this CVE path.

3. **Wire it into setup**
   - Update `src/bonesinfra/deploys/setup/plan.py`.
   - Import `kernel_hardening`.
   - Call it during setup after base packages are installed.
   - Reason: `/etc/modprobe.d` exists on normal systems, but doing it after setup packages keeps the provisioning story simple.

4. **Add the smallest structural test**
   - Update existing test file, likely `tests/test_deploy_structure.py`.
   - Assert:
     - setup plan calls `kernel_hardening.configure`
     - `kernel_hardening.py` writes `/etc/modprobe.d/disable-algif.conf`
     - asset contains `install algif_aead /bin/false`
   - No integration test, no mocking pyinfra, no real module manipulation.

5. **Do not include these in the minimum fix**
   - No seccomp.
   - No AppArmor changes.
   - No Laravel PHP-FPM confinement.
   - No automatic reboot.
   - No broad SSH hardening.
   - No sysctl hardening.
   - No `apt upgrade` in this specific patch unless you want to address Lynis package warnings at the same time.

6. **Optional documentation**
   - Add one sentence to `docs/PROJECT.md` under setup provisioning:
     - setup disables `algif_aead` via modprobe as a defense-in-depth mitigation for Copy Fail-style `AF_ALG` kernel LPEs.
   - If you want the absolute smallest code-only fix, skip docs.

7. **Verification**
   - Run:
     ```sh
     ruff check .
     ruff format .
     uv run pytest
     ```

**Why this is enough**

Copy Fail needs `algif_aead`. Preventing that module from loading, and removing it if already loaded, directly breaks the exploit path. This is the advisory’s own pre-patch mitigation and is much smaller than trying to solve kernel patch orchestration, seccomp, or full runtime confinement.