abbrev Hash := Nat

def GENESIS_PREV_HASH : Hash := 0

structure Block where
  hash : Hash
  height : Nat
  prev_hash : Hash
deriving Repr

def Block.is_genesis (b : Block) : Bool :=
  b.prev_hash == GENESIS_PREV_HASH

axiom Block.hash_unique : ∀ (a b : Block), a.hash = b.hash → a = b

def genesis : Block :=
  { hash := 0
    height := 0
    prev_hash := GENESIS_PREV_HASH }

inductive Chain : List Block → Prop where
  | nil : Chain []
  | single : ∀ b, Chain [b]
  | cons : ∀ b₁ b₂ bs, b₂.prev_hash = b₁.hash → Chain (b₂ :: bs) → Chain (b₁ :: b₂ :: bs)

inductive RevChain : List Block → Prop where
  | nil : RevChain []
  | single : ∀ b, RevChain [b]
  | cons : ∀ b₁ b₂ bs, b₁.prev_hash = b₂.hash → RevChain (b₂ :: bs) → RevChain (b₁ :: b₂ :: bs)

-- Fork: a Chain that starts from genesis (first block in list has prev_hash = GENESIS_PREV_HASH)
inductive Fork : List Block → Prop where
  | mk : ∀ b bs, b.prev_hash = GENESIS_PREV_HASH → Chain (b :: bs) → Fork (b :: bs)

structure V where
  fork : List Block
  is_fork : Fork fork
  time : Nat
  epoch : Nat
deriving Repr

-- Initial state: genesis block only
def fork_genesis : Fork [genesis] :=
  Fork.mk genesis [] (by simp [genesis]) (Chain.single genesis)

def V₀ : V :=
  { fork := [genesis]
    is_fork := fork_genesis
    time := 0
    epoch := 0 }

-- Return the tip (last block) of a list
def tip (bs : List Block) : Block :=
  match bs with
  | [] => genesis
  | b :: [] => b
  | _ :: b :: bs => tip (b :: bs)

-- Extend a Chain with a block that links to its tip
theorem chain_extend {cs : List Block} {b : Block} (hc : Chain cs) (hlink : b.prev_hash = (tip cs).hash) : Chain (cs ++ [b]) := by
  induction hc with
  | nil =>
    exact Chain.single b
  | single b₁ =>
    exact Chain.cons b₁ b [] hlink (Chain.single b)
  | cons b₁ b₂ bs h12 hrest ih =>
    have htip : tip (b₁ :: b₂ :: bs) = tip (b₂ :: bs) := by simp [tip]
    have hlink' : b.prev_hash = (tip (b₂ :: bs)).hash := by rw [← htip, hlink]
    have hrest' : Chain ((b₂ :: bs) ++ [b]) := ih hlink'
    exact Chain.cons b₁ b₂ (bs ++ [b]) h12 hrest'

-- Extend a Fork with a block that links to its tip
theorem fork_extend {bs : List Block} {b : Block} (hf : Fork bs) (h : b.prev_hash = (tip bs).hash) : Fork (bs ++ [b]) := by
  cases hf with
  | mk g rest hgen hchain =>
    apply Fork.mk g (rest ++ [b]) hgen
    apply chain_extend hchain h

-- Trim the fork after the block whose hash matches `target` (inclusive — keeps the match, drops the rest)
def trimAfter (bs : List Block) (target : Hash) : List Block :=
  match bs with
  | [] => []
  | b :: rest =>
    if b.hash = target then [b]
    else b :: trimAfter rest target

-- trimAfter on a non-empty list always yields a list starting with the same head
theorem trimAfter_cons_eq {b : Block} {bs : List Block} {target : Hash} :
    ∃ bs', trimAfter (b :: bs) target = b :: bs' := by
  unfold trimAfter
  split
  · exact ⟨[], rfl⟩
  · exact ⟨trimAfter bs target, rfl⟩

-- Trimming a Chain at an existing hash yields a Chain
theorem chain_trim {cs : List Block} {target : Hash} (hc : Chain cs) (h : target ∈ cs.map Block.hash) : Chain (trimAfter cs target) := by
  induction hc with
  | nil =>
    simp at h
  | single b₁ =>
    simp at h
    subst h
    unfold trimAfter; simp
    exact Chain.single b₁
  | cons b₁ b₂ bs h12 hrest ih =>
    simp at h
    rcases h with (heq₁ | hmem)
    · subst heq₁; unfold trimAfter; simp; exact Chain.single b₁
    · have hmem' : target ∈ (b₂ :: bs).map Block.hash := by simpa using hmem
      by_cases heq : b₁.hash = target
      · subst heq; unfold trimAfter; simp; exact Chain.single b₁
      · unfold trimAfter; simp [heq]
        have htrim : Chain (trimAfter (b₂ :: bs) target) := ih hmem'
        rcases trimAfter_cons_eq (b := b₂) (bs := bs) (target := target) with ⟨bs', h_eq⟩
        rw [h_eq]
        rw [h_eq] at htrim
        exact Chain.cons b₁ b₂ bs' h12 htrim

-- Trimming a Fork at an existing hash yields a Fork
theorem fork_trim {bs : List Block} {target : Hash} (hf : Fork bs) (h : target ∈ bs.map Block.hash) : Fork (trimAfter bs target) := by
  cases hf with
  | mk g rest hgen hchain =>
    simp at h
    rcases h with (heq | hmem)
    · subst heq; unfold trimAfter; simp
      exact Fork.mk g [] hgen (Chain.single g)
    · by_cases heq_g : g.hash = target
      · subst heq_g; unfold trimAfter; simp; exact Fork.mk g [] hgen (Chain.single g)
      · unfold trimAfter; simp [heq_g]
        have hmem_full : target ∈ (g :: rest).map Block.hash := by
          simp [hmem]
        apply Fork.mk g (trimAfter rest target) hgen
        simpa [trimAfter, heq_g] using chain_trim hchain hmem_full

-- The tip of trimAfter has hash equal to the target (when target is in the list)
-- trimAfter on a non-empty list returns a non-empty list
theorem trimAfter_ne_nil {b : Block} {bs : List Block} {target : Hash} : trimAfter (b :: bs) target ≠ [] := by
  unfold trimAfter; split <;> simp

-- The tip of trimAfter has hash equal to the target (when target is in the list)
theorem trimAfter_tip_hash {bs : List Block} {target : Hash} (h : target ∈ bs.map Block.hash) : (tip (trimAfter bs target)).hash = target := by
  induction bs with
  | nil => simp at h
  | cons b bs ih =>
    simp [List.map] at h
    unfold trimAfter
    by_cases h_eq : b.hash = target
    · -- then branch: trimAfter = [b]
      simp [h_eq, tip]
    · -- else branch: trimAfter = b :: trimAfter bs target
      simp [h_eq]
      rcases h with (heq | hmem)
      · exfalso; exact h_eq (by symm; exact heq)
      · -- goal: (tip (b :: trimAfter bs target)).hash = target
        -- trimAfter bs target ≠ [] because target ∈ bs.map hash
        have h_ne_nil : trimAfter bs target ≠ [] := by
          -- prove by showing it's of the form trimAfter (b' :: bs') for some b', bs'
          -- Since target ∈ bs.map hash, bs is non-empty
          cases bs
          · simp at hmem
          · exact trimAfter_ne_nil
        -- Now tip (b :: nonempty) = tip nonempty
        have htip : tip (b :: trimAfter bs target) = tip (trimAfter bs target) := by
          rcases trimAfter bs target with (⟨⟩ | b' :: bs')
          · exact absurd rfl h_ne_nil
          · simp [tip]
        rw [htip]
        exact ih hmem

axiom addBlock_links : ∀ (v : V) (b : Block), b.prev_hash ∈ (v.fork.map Block.hash)

-- Add a block to the fork. Trims to the ancestor, then extends.
def addBlock (b : Block) : StateM V Unit := do
  let v ← get
  let trimmed := trimAfter v.fork b.prev_hash
  let fork' := trimmed ++ [b]
  have hfork' : Fork fork' := by
    have hlink := addBlock_links v b
    have htrim_fork : Fork trimmed := fork_trim v.is_fork hlink
    have htip : (tip trimmed).hash = b.prev_hash := trimAfter_tip_hash hlink
    have hlink_tip : b.prev_hash = (tip trimmed).hash := by rw [htip]
    exact fork_extend htrim_fork hlink_tip
  set { v with fork := fork', is_fork := hfork' }
