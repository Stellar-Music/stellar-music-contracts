# Stellar Music Protocol — Mathematical Specification of Integer Stroop Arithmetic

## 1. Abstract

In digital music monetization, multi-party revenue distribution frequently suffers from fractional cent truncation and floating-point precision loss. In traditional streaming systems, rounding dust is often silently absorbed into platform operator balance sheets.

**Stellar Music** solves this via a deterministic, integer-only financial accounting engine operating at the Stellar native base unit: the **Stroop** ($1\text{ XLM} = 10^7\text{ stroops} = 10,000,000\text{ stroops}$).

---

## 2. Definitions & Units

| Term | Symbol | Definition |
| :--- | :--- | :--- |
| **Lumen (XLM)** | $\text{XLM}$ | Stellar Network native asset |
| **Stroop** | $\text{str}$ | $10^{-7}\text{ XLM}$ ($1\text{ XLM} = 10,000,000\text{ stroops}$) |
| **Basis Points** | $\text{bps}$ | $1\text{ bps} = 0.01\% = 10^{-4}$; $10,000\text{ bps} = 100.00\%$ |
| **Gross Pool** | $S_{\text{gross}}$ | Total revenue stroops available for distribution ($S_{\text{gross}} \in \mathbb{N}^+$) |
| **Recipient Share** | $b_i$ | Contributor basis points allocation ($b_i \in \mathbb{N}^+$) |
| **Collaborator Count** | $k$ | Number of valid split recipients ($1 \le k \le 25$) |

---

## 3. The 10,000 Basis Points Completeness Axiom

A split agreement is mathematically valid if and only if the sum of all contributor basis points strictly equals 10,000:

$$\sum_{i=0}^{k-1} b_i = 10,000$$

Any split proposal where $\sum b_i \ne 10,000$ is rejected on-chain with `Error::InvalidBasisPoints`.

---

## 4. Deterministic Floor Allocation & Remainder Dust Proof

For each contributor $i \in \{0, \dots, k-1\}$, the initial base allocation is computed via integer floor division:

$$\text{Allocated}_i = \left\lfloor \frac{S_{\text{gross}} \cdot b_i}{10,000} \right\rfloor$$

The total allocated stroops across all parties prior to remainder adjustment is:

$$S_{\text{allocated}} = \sum_{i=0}^{k-1} \text{Allocated}_i = \sum_{i=0}^{k-1} \left\lfloor \frac{S_{\text{gross}} \cdot b_i}{10,000} \right\rfloor$$

### Remainder Dust ($R$)
The unallocated fractional dust $R$ is defined as:

$$R = S_{\text{gross}} - S_{\text{allocated}}$$

**Theorem 1 (Non-Negativity):** $R \ge 0$.  
*Proof:* Since for all real $x$, $\lfloor x \rfloor \le x$, we have:
$$S_{\text{allocated}} = \sum_{i=0}^{k-1} \left\lfloor \frac{S_{\text{gross}} \cdot b_i}{10,000} \right\rfloor \le \sum_{i=0}^{k-1} \frac{S_{\text{gross}} \cdot b_i}{10,000} = \frac{S_{\text{gross}}}{10,000} \sum_{i=0}^{k-1} b_i = \frac{S_{\text{gross}}}{10,000} \cdot 10,000 = S_{\text{gross}}$$
Thus, $S_{\text{gross}} - S_{\text{allocated}} \ge 0$, proving $R \ge 0$. $\blacksquare$

**Theorem 2 (Bounded Dust Ceiling):** $R < k \le 25\text{ stroops}$.  
*Proof:* The fractional loss for each term is $x - \lfloor x \rfloor < 1$. Summing across $k$ terms:
$$R = \sum_{i=0}^{k-1} \left( \frac{S_{\text{gross}} \cdot b_i}{10,000} - \left\lfloor \frac{S_{\text{gross}} \cdot b_i}{10,000} \right\rfloor \right) < \sum_{i=0}^{k-1} 1 = k$$
Since $k \le 25$, $R \le 24\text{ stroops}$ ($< 0.0000024\text{ XLM}$). $\blacksquare$

---

## 5. Zero-Loss Remainder Absorption Policy

To guarantee the strict mathematical conservation of value, all remainder dust $R$ is assigned deterministically to the primary creator (index $i = 0$):

$$\text{FinalPayout}_0 = \text{Allocated}_0 + R$$
$$\text{FinalPayout}_i = \text{Allocated}_i \quad \forall i \in \{1, \dots, k-1\}$$

### Conservation Invariant Proof:
$$\sum_{i=0}^{k-1} \text{FinalPayout}_i = (\text{Allocated}_0 + R) + \sum_{i=1}^{k-1} \text{Allocated}_i = \sum_{i=0}^{k-1} \text{Allocated}_i + R = S_{\text{allocated}} + (S_{\text{gross}} - S_{\text{allocated}}) \equiv S_{\text{gross}}$$

**Conclusion**: Exactly $0\text{ stroops}$ are lost, created, or leaked across any number of contributors or financial distributions.

---

## 6. Empirical Verification Summary

The protocol mathematical invariants are continuously validated by dual independent test harnesses:

1. **Smart Contracts Harness (`src/test.rs`)**: On-chain Soroban integer test over diverse basis points and edge amounts ($1\text{ str}$ to $100\text{B str}$).
2. **Backend Engine Harness (`tests/stroop_math_invariant.test.ts`)**: 10,000 Monte Carlo randomized multi-party split executions with 100.00% invariant adherence.
