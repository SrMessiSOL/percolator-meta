# FILING_PLAN

Do not run these commands from this agent session. They are for the user to run manually.

## 1. Create Issues

```bash
gh issue create --repo aeyakovenko/percolator-meta --title "Governance adapter permits arbitrary market reward initialization" --body-file ISSUE_F1_BODY.md
```

Save the created number as `F1_ISSUE`.

```bash
gh issue create --repo aeyakovenko/percolator-meta --title "pull_insurance signs rewards vault authority into caller-supplied program" --body-file ISSUE_F2_BODY.md
```

Save the created number as `F2_ISSUE`.

```bash
gh issue create --repo aeyakovenko/percolator-meta --title "Governance adapter permits arbitrary profit draws from reward vaults" --body-file ISSUE_F3_BODY.md
```

Save the created number as `F3_ISSUE`.

```bash
gh issue create --repo aeyakovenko/percolator-meta --title "init_market_rewards accepts non-Percolator slabs" --body-file ISSUE_O1_BODY.md
```

Save the created number as `O1_ISSUE`.

## 2. Fill PR Closes Lines

Replace the placeholders in `PR_BODY.md`:

```text
Closes #<TBD-F1>
Closes #<TBD-F2>
Closes #<TBD-F3>
Closes #<TBD-O1>
```

with the real issue numbers from step 1.

## 3. Push Branch

Confirm the branch is still `fix/critical-findings` and based on `origin/master`:

```bash
git status
git log --oneline --decorate origin/master..HEAD
```

Then push:

```bash
git push -u origin fix/critical-findings
```

## 4. Open PR

```bash
gh pr create --repo aeyakovenko/percolator-meta --base master --head fix/critical-findings --title "Fix governance adapter and Percolator CPI authority gaps" --body-file PR_BODY.md
```

## 5. Follow Up on Issue #1

After the issue numbers and PR number exist, replace the placeholders in `ISSUE_1_FOLLOWUP.md`, then post that text manually on issue #1.
