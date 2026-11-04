# Golden-sample source excerpts (KJV, BIB-KJV-CW31)

**Machine-extracted. Do not edit by hand.** Every excerpt below was copied by a script from the pinned source file, byte for byte, from the first byte of the verse's `<verse … sID=…/>` tag to the last byte of its `<verse eID=…/>` tag. Nothing here was typed by a person, and nothing was normalised, re-wrapped or re-encoded.

| Field | Value |
|---|---|
| Source ID | BIB-KJV-CW31 (`sources/SOURCES.md`) |
| Source file | `sources/raw/BIB-KJV-CW31/kjv.osis.xml` |
| Pinned URL | https://gitlab.com/crosswire-bible-society/kjv/-/raw/d490be7e34762deb2c76cb2c1306d4808e27890d/kjv.osis.xml |
| Size (bytes) | 28043075 |
| SHA-256 | `4bcbfe45722fca396adbeac9c81025bbb173cb1e30296da90864f0400cb0e3f5` |
| Git blob SHA-1 | `369705ecfc1f734202bdb8c195fd1c930f9f89d2` (checked with `git hash-object`) |
| References | the 100 distinct `ref:` values in the YAML block of `docs/scholar/red-letter-golden-samples.md`, in that order |
| Generator | `docs/pipeline/survey-scripts/golden_excerpts.py` (regex tokenizer, no XML parser; checks the SHA-256 first) |

## How to use this file

- This is the **source** column for the golden samples: what the pinned digital file actually marks. A person compares it with the printed edition (the scholar's file) and fills `source_expected` by hand from these excerpts.
- Red letters are the `<q who="Jesus" marker="">` … `</q>` containers. They are not milestones, so a span that started in an earlier verse has its start tag **outside** this excerpt. The line "`q` open at verse start" says so, and names the verse where that `<q>` tag is. Those two lines and the tag counts are machine-derived from the same tokenizer.
- `<milestone type="x-p" marker="¶"/>` is a paragraph mark, not text. `<note>` content (KJV margin notes) is not verse text and is not imported in M1.
- Each excerpt (`xml` block, authoritative) is followed by a **reading aid** (`text` block): the same bytes with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags, so the `<q>` boundaries are easier to see. The generator checks that the aid's text content (everything outside tags) is identical to the excerpt's. When in doubt, read the `xml` block.
- To re-check an excerpt's bytes: take `length` bytes starting at byte `offset` (0-based) of the source file and compare the SHA-256 with the one given.
- Excerpts are kept to the golden references only (internal review document; the text is under BIB-KJV-CW31's terms).

### MAT.3.15 (OSIS `Matt.3.15`)

- offset 15263273, length 1818, excerpt SHA-256 `6727fdddfcb7c2aef8b0a2c2677d99d0c090d15d34a0fd4df56fc8b9b8f549ca`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 1; `</q>` end tags in excerpt: 1

```xml
<verse osisID="Matt.3.15" sID="Matt.3.15"/><w src="2" lemma="strong:G1161 lemma.TR:δε" morph="robinson:CONJ">And</w> <w src="3 4" lemma="strong:G3588 strong:G2424 lemma.TR:ο lemma.TR:ιησους" morph="robinson:T-NSM robinson:N-NSM">Jesus</w> <w src="1" lemma="strong:G611 lemma.TR:αποκριθεις" morph="robinson:V-AOP-NSM">answering</w> <w src="5" lemma="strong:G3004 lemma.TR:ειπεν" morph="robinson:V-2AAI-3S">said</w> <w src="6" lemma="strong:G4314 lemma.TR:προς" morph="robinson:PREP">unto</w> <w src="7" lemma="strong:G846 lemma.TR:αυτον" morph="robinson:P-ASM">him</w>, <q who="Jesus" marker=""><w src="8" lemma="strong:G863 lemma.TR:αφες" morph="robinson:V-2AAM-2S">Suffer</w> <transChange type="added">it to be so</transChange> <w src="9" lemma="strong:G737 lemma.TR:αρτι" morph="robinson:ADV">now</w>: <w src="11" lemma="strong:G1063 lemma.TR:γαρ" morph="robinson:CONJ">for</w> <w src="10" lemma="strong:G3779 lemma.TR:ουτως" morph="robinson:ADV">thus</w> <w src="12 13 " lemma="strong:G4241 strong:G1510 lemma.TR:πρεπον lemma.TR:εστιν" morph="robinson:V-PAP-NSN robinson:V-PAI-3S">it becometh</w> <w src="14" lemma="strong:G1473 lemma.TR:ημιν" morph="robinson:P-1DP">us</w> <w src="15" lemma="strong:G4137 lemma.TR:πληρωσαι" morph="robinson:V-AAN">to fulfil</w> <w src="16" lemma="strong:G3956 lemma.TR:πασαν" morph="robinson:A-ASF">all</w> <w src="17" lemma="strong:G1343 lemma.TR:δικαιοσυνην" morph="robinson:N-ASF">righteousness</w>.</q> <w src="18" lemma="strong:G5119 lemma.TR:τοτε" morph="robinson:ADV">Then</w> <w src="19" lemma="strong:G863 lemma.TR:αφιησιν" morph="robinson:V-PAI-3S">he suffered</w> <w src="20" lemma="strong:G846 lemma.TR:αυτον" morph="robinson:P-ASM">him</w>.<verse eID="Matt.3.15"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Matt.3.15" sID="Matt.3.15"/><w>And</w> <w>Jesus</w> <w>answering</w> <w>said</w> <w>unto</w> <w>him</w>, <q who="Jesus" marker=""><w>Suffer</w> <transChange type="added">it to be so</transChange> <w>now</w>: <w>for</w> <w>thus</w> <w>it becometh</w> <w>us</w> <w>to fulfil</w> <w>all</w> <w>righteousness</w>.</q> <w>Then</w> <w>he suffered</w> <w>him</w>.<verse eID="Matt.3.15"/>
```

### MAT.3.17 (OSIS `Matt.3.17`)

- offset 15267442, length 1418, excerpt SHA-256 `071a13da5355ccb0c41fd183f18945a3ec35a4088f5819736a23e63e44db1571`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="Matt.3.17" sID="Matt.3.17"/><w src="13" lemma="strong:G3588 lemma.TR:ο" morph="robinson:T-NSM"/><w src="1" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">And</w> <w src="2" lemma="strong:G3708 lemma.TR:ιδου" morph="robinson:V-2AMM-2S">lo</w> <w src="3" lemma="strong:G5456 lemma.TR:φωνη" morph="robinson:N-NSF">a voice</w> <w src="4" lemma="strong:G1537 lemma.TR:εκ" morph="robinson:PREP">from</w> <w src="5 6" lemma="strong:G3588 strong:G3772 lemma.TR:των lemma.TR:ουρανων" morph="robinson:T-GPM robinson:N-GPM">heaven</w>, <w src="7" lemma="strong:G3004 lemma.TR:λεγουσα" morph="robinson:V-PAP-NSF">saying</w>, <w src="8" lemma="strong:G3778 lemma.TR:ουτος" morph="robinson:D-NSM">This</w> <w src="9" lemma="strong:G1510 lemma.TR:εστιν" morph="robinson:V-PAI-3S">is</w> <w src="12" lemma="strong:G1473 lemma.TR:μου" morph="robinson:P-1GS">my</w> <w src="14" lemma="strong:G27 lemma.TR:αγαπητος" morph="robinson:A-NSM">beloved</w> <w src="10 11" lemma="strong:G3588 strong:G5207 lemma.TR:ο lemma.TR:υιος" morph="robinson:T-NSM robinson:N-NSM">Son</w>, <w src="15" lemma="strong:G1722 lemma.TR:εν" morph="robinson:PREP">in</w> <w src="16" lemma="strong:G3739 lemma.TR:ω" morph="robinson:R-DSM">whom</w> <w src="17" lemma="strong:G2106 lemma.TR:ευδοκησα" morph="robinson:V-AAI-1S">I am well pleased</w>.<verse eID="Matt.3.17"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Matt.3.17" sID="Matt.3.17"/><w/><w>And</w> <w>lo</w> <w>a voice</w> <w>from</w> <w>heaven</w>, <w>saying</w>, <w>This</w> <w>is</w> <w>my</w> <w>beloved</w> <w>Son</w>, <w>in</w> <w>whom</w> <w>I am well pleased</w>.<verse eID="Matt.3.17"/>
```

### MAT.4.4 (OSIS `Matt.4.4`)

- offset 15272845, length 1877, excerpt SHA-256 `cd43f02021d5fc0be14e6122229a8425d2e18c6ce298f11dcb88476cae6eb998`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 1; `</q>` end tags in excerpt: 1

```xml
<verse osisID="Matt.4.4" sID="Matt.4.4"/><w src="1" lemma="strong:G3588 lemma.TR:ο" morph="robinson:T-NSM"/><w src="2" lemma="strong:G1161 lemma.TR:δε" morph="robinson:CONJ">But</w> <w src="3" lemma="strong:G611 lemma.TR:αποκριθεις" morph="robinson:V-AOP-NSM">he answered</w> <w src="4" lemma="strong:G3004 lemma.TR:ειπεν" morph="robinson:V-2AAI-3S">and said</w>, <q who="Jesus" marker=""><w src="5" lemma="strong:G1125 lemma.TR:γεγραπται" morph="robinson:V-RPI-3S">It is written</w>, <w src="11" lemma="strong:G444 lemma.TR:ανθρωπος" morph="robinson:N-NSM">Man</w> <w src="10" lemma="strong:G2198 lemma.TR:ζησεται" morph="robinson:V-FDI-3S" type="x-split-410">shall</w> <w src="6" lemma="strong:G3756 lemma.TR:ουκ" morph="robinson:PRT-N">not</w> <w src="10" lemma="strong:G2198 lemma.TR:ζησεται" morph="robinson:V-FDI-3S" type="x-split-410">live</w> <w src="7" lemma="strong:G1909 lemma.TR:επ" morph="robinson:PREP">by</w> <w src="8" lemma="strong:G740 lemma.TR:αρτω" morph="robinson:N-DSM">bread</w> <w src="9" lemma="strong:G3441 lemma.TR:μονω" morph="robinson:A-DSM">alone</w>, <w src="12" lemma="strong:G235 lemma.TR:αλλ" morph="robinson:CONJ">but</w> <w src="13" lemma="strong:G1909 lemma.TR:επι" morph="robinson:PREP">by</w> <w src="14" lemma="strong:G3956 lemma.TR:παντι" morph="robinson:A-DSN">every</w> <w src="15" lemma="strong:G4487 lemma.TR:ρηματι" morph="robinson:N-DSN">word</w> <w src="16" lemma="strong:G1607 lemma.TR:εκπορευομενω" morph="robinson:V-PNP-DSN">that proceedeth</w> <w src="17" lemma="strong:G1223 lemma.TR:δια" morph="robinson:PREP">out</w> <w src="18" lemma="strong:G4750 lemma.TR:στοματος" morph="robinson:N-GSN">of the mouth</w> <w src="19" lemma="strong:G2316 lemma.TR:θεου" morph="robinson:N-GSM">of God</w>.</q><verse eID="Matt.4.4"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Matt.4.4" sID="Matt.4.4"/><w/><w>But</w> <w>he answered</w> <w>and said</w>, <q who="Jesus" marker=""><w>It is written</w>, <w>Man</w> <w>shall</w> <w>not</w> <w>live</w> <w>by</w> <w>bread</w> <w>alone</w>, <w>but</w> <w>by</w> <w>every</w> <w>word</w> <w>that proceedeth</w> <w>out</w> <w>of the mouth</w> <w>of God</w>.</q><verse eID="Matt.4.4"/>
```

### MAT.4.6 (OSIS `Matt.4.6`)

- offset 15276295, length 3029, excerpt SHA-256 `37c2028127960f284d8c3231eda5d87167fce0564f65be94a3b85684f81b0ea0`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="Matt.4.6" sID="Matt.4.6"/><w src="1" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">And</w> <w src="2" lemma="strong:G3004 lemma.TR:λεγει" morph="robinson:V-PAI-3S">saith</w> <w src="3" lemma="strong:G846 lemma.TR:αυτω" morph="robinson:P-DSM">unto him</w>, <w src="4" lemma="strong:G1487 lemma.TR:ει" morph="robinson:COND">If</w> <w src="6" lemma="strong:G1510 lemma.TR:ει" morph="robinson:V-PAI-2S">thou be</w> <w src="5" lemma="strong:G5207 lemma.TR:υιος" morph="robinson:N-NSM">the Son</w> <w src="7 8" lemma="strong:G3588 strong:G2316 lemma.TR:του lemma.TR:θεου" morph="robinson:T-GSM robinson:N-GSM">of God</w>, <w src="9" lemma="strong:G906 lemma.TR:βαλε" morph="robinson:V-2AAM-2S">cast</w> <w src="10" lemma="strong:G4572 lemma.TR:σεαυτον" morph="robinson:F-2ASM">thyself</w> <w src="11" lemma="strong:G2736 lemma.TR:κατω" morph="robinson:ADV">down</w>: <w src="13" lemma="strong:G1063 lemma.TR:γαρ" morph="robinson:CONJ">for</w> <w src="12" lemma="strong:G1125 lemma.TR:γεγραπται" morph="robinson:V-RPI-3S">it is written</w>, <w src="14" lemma="strong:G3754 lemma.TR:οτι" morph="robinson:CONJ"/><w src="18" lemma="strong:G1781 lemma.TR:εντελειται" morph="robinson:V-FNI-3S" type="x-split-412">He shall give</w> <w src="17" lemma="strong:G846 lemma.TR:αυτου" morph="robinson:P-GSM">his</w> <w src="15 16" lemma="strong:G3588 strong:G32 lemma.TR:τοις lemma.TR:αγγελοις" morph="robinson:T-DPM robinson:N-DPM">angels</w> <w src="18" lemma="strong:G1781 lemma.TR:εντελειται" morph="robinson:V-FNI-3S" type="x-split-412">charge</w> <w src="19" lemma="strong:G4012 lemma.TR:περι" morph="robinson:PREP">concerning</w> <w src="20" lemma="strong:G4771 lemma.TR:σου" morph="robinson:P-2GS">thee</w>: <w src="21" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="22" lemma="strong:G1909 lemma.TR:επι" morph="robinson:PREP">in</w> <transChange type="added">their</transChange> <w src="23" lemma="strong:G5495 lemma.TR:χειρων" morph="robinson:N-GPF">hands</w> <w src="24" lemma="strong:G142 lemma.TR:αρουσιν" morph="robinson:V-FAI-3P" type="x-split-413">they shall bear</w> <w src="25" lemma="strong:G4771 lemma.TR:σε" morph="robinson:P-2AS">thee</w> <w src="24" lemma="strong:G142 lemma.TR:αρουσιν" morph="robinson:V-FAI-3P" type="x-split-413">up</w>, <w src="26" lemma="strong:G3379 lemma.TR:μηποτε" morph="robinson:ADV-N">lest at any time</w> <w src="27" lemma="strong:G4350 lemma.TR:προσκοψης" morph="robinson:V-AAS-2S">thou dash</w> <w src="32" lemma="strong:G4771 lemma.TR:σου" morph="robinson:P-2GS">thy</w> <w src="30 31" lemma="strong:G3588 strong:G4228 lemma.TR:τον lemma.TR:ποδα" morph="robinson:T-ASM robinson:N-ASM">foot</w> <w src="28" lemma="strong:G4314 lemma.TR:προς" morph="robinson:PREP">against</w> <w src="29" lemma="strong:G3037 lemma.TR:λιθον" morph="robinson:N-ASM">a stone</w>.<verse eID="Matt.4.6"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Matt.4.6" sID="Matt.4.6"/><w>And</w> <w>saith</w> <w>unto him</w>, <w>If</w> <w>thou be</w> <w>the Son</w> <w>of God</w>, <w>cast</w> <w>thyself</w> <w>down</w>: <w>for</w> <w>it is written</w>, <w/><w>He shall give</w> <w>his</w> <w>angels</w> <w>charge</w> <w>concerning</w> <w>thee</w>: <w>and</w> <w>in</w> <transChange type="added">their</transChange> <w>hands</w> <w>they shall bear</w> <w>thee</w> <w>up</w>, <w>lest at any time</w> <w>thou dash</w> <w>thy</w> <w>foot</w> <w>against</w> <w>a stone</w>.<verse eID="Matt.4.6"/>
```

### MAT.4.7 (OSIS `Matt.4.7`)

- offset 15279325, length 1200, excerpt SHA-256 `1d0967d4014140e426d10ff9a3aaccc6c043516335867f749a5735e688251ada`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 1; `</q>` end tags in excerpt: 1

```xml
<verse osisID="Matt.4.7" sID="Matt.4.7"/><w src="3 4" lemma="strong:G3588 strong:G2424 lemma.TR:ο lemma.TR:ιησους" morph="robinson:T-NSM robinson:N-NSM">Jesus</w> <w src="1" lemma="strong:G5346 lemma.TR:εφη" morph="robinson:V-IAI-3S">said</w> <w src="2" lemma="strong:G846 lemma.TR:αυτω" morph="robinson:P-DSM">unto him</w>, <q who="Jesus" marker=""><w src="6" lemma="strong:G1125 lemma.TR:γεγραπται" morph="robinson:V-RPI-3S">It is written</w> <w src="5" lemma="strong:G3825 lemma.TR:παλιν" morph="robinson:ADV">again</w>, <w src="8" lemma="strong:G1598 lemma.TR:εκπειρασεις" morph="robinson:V-FAI-2S" type="x-split-414">Thou shalt</w> <w src="7" lemma="strong:G3756 lemma.TR:ουκ" morph="robinson:PRT-N">not</w> <w src="8" lemma="strong:G1598 lemma.TR:εκπειρασεις" morph="robinson:V-FAI-2S" type="x-split-414">tempt</w> <w src="9" lemma="strong:G2962 lemma.TR:κυριον" morph="robinson:N-ASM">the Lord</w> <w src="12" lemma="strong:G4771 lemma.TR:σου" morph="robinson:P-2GS">thy</w> <w src="10 11" lemma="strong:G3588 strong:G2316 lemma.TR:τον lemma.TR:θεον" morph="robinson:T-ASM robinson:N-ASM">God</w>.</q><verse eID="Matt.4.7"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Matt.4.7" sID="Matt.4.7"/><w>Jesus</w> <w>said</w> <w>unto him</w>, <q who="Jesus" marker=""><w>It is written</w> <w>again</w>, <w>Thou shalt</w> <w>not</w> <w>tempt</w> <w>the Lord</w> <w>thy</w> <w>God</w>.</q><verse eID="Matt.4.7"/>
```

### MAT.4.10 (OSIS `Matt.4.10`)

- offset 15283468, length 1618, excerpt SHA-256 `cf9caac0b10b0086d424872bea230106dd389102c64947665c74049c015603db`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 1; `</q>` end tags in excerpt: 1

```xml
<verse osisID="Matt.4.10" sID="Matt.4.10"/><w src="1" lemma="strong:G5119 lemma.TR:τοτε" morph="robinson:ADV">Then</w> <w src="2" lemma="strong:G3004 lemma.TR:λεγει" morph="robinson:V-PAI-3S">saith</w> <w src="4 5" lemma="strong:G3588 strong:G2424 lemma.TR:ο lemma.TR:ιησους" morph="robinson:T-NSM robinson:N-NSM">Jesus</w> <w src="3" lemma="strong:G846 lemma.TR:αυτω" morph="robinson:P-DSM">unto him</w>, <q who="Jesus" marker=""><w src="6" lemma="strong:G5217 lemma.TR:υπαγε" morph="robinson:V-PAM-2S">Get thee hence</w>, <w src="7" lemma="strong:G4567 lemma.TR:σατανα" morph="robinson:N-VSM">Satan</w>: <w src="9" lemma="strong:G1063 lemma.TR:γαρ" morph="robinson:CONJ">for</w> <w src="8" lemma="strong:G1125 lemma.TR:γεγραπται" morph="robinson:V-RPI-3S">it is written</w>, <w src="14" lemma="strong:G4352 lemma.TR:προσκυνησεις" morph="robinson:V-FAI-2S">Thou shalt worship</w> <w src="10" lemma="strong:G2962 lemma.TR:κυριον" morph="robinson:N-ASM">the Lord</w> <w src="13" lemma="strong:G4771 lemma.TR:σου" morph="robinson:P-2GS">thy</w> <w src="11 12" lemma="strong:G3588 strong:G2316 lemma.TR:τον lemma.TR:θεον" morph="robinson:T-ASM robinson:N-ASM">God</w>, <w src="15" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="16" lemma="strong:G846 lemma.TR:αυτω" morph="robinson:P-DSM">him</w> <w src="17" lemma="strong:G3441 lemma.TR:μονω" morph="robinson:A-DSM">only</w> <w src="18" lemma="strong:G3000 lemma.TR:λατρευσεις" morph="robinson:V-FAI-2S">shalt thou serve</w>.</q><verse eID="Matt.4.10"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Matt.4.10" sID="Matt.4.10"/><w>Then</w> <w>saith</w> <w>Jesus</w> <w>unto him</w>, <q who="Jesus" marker=""><w>Get thee hence</w>, <w>Satan</w>: <w>for</w> <w>it is written</w>, <w>Thou shalt worship</w> <w>the Lord</w> <w>thy</w> <w>God</w>, <w>and</w> <w>him</w> <w>only</w> <w>shalt thou serve</w>.</q><verse eID="Matt.4.10"/>
```

### MAT.4.17 (OSIS `Matt.4.17`)

- offset 15292401, length 1353, excerpt SHA-256 `9552ebf32c3a52b320dd9437d9509bb872cb572306abb1b857231b0db613739a`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 1; `</q>` end tags in excerpt: 1

```xml
<verse osisID="Matt.4.17" sID="Matt.4.17"/><milestone type="x-p" subType="x-added" marker="¶"/><w src="1" lemma="strong:G575 lemma.TR:απο" morph="robinson:PREP">From</w> <w src="2" lemma="strong:G5119 lemma.TR:τοτε" morph="robinson:ADV">that time</w> <w src="4 5" lemma="strong:G3588 strong:G2424 lemma.TR:ο lemma.TR:ιησους" morph="robinson:T-NSM robinson:N-NSM">Jesus</w> <w src="3" lemma="strong:G756 lemma.TR:ηρξατο" morph="robinson:V-ADI-3S">began</w> <w src="6" lemma="strong:G2784 lemma.TR:κηρυσσειν" morph="robinson:V-PAN">to preach</w>, <w src="7" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="8" lemma="strong:G3004 lemma.TR:λεγειν" morph="robinson:V-PAN">to say</w>, <q who="Jesus" marker=""><w src="9" lemma="strong:G3340 lemma.TR:μετανοειτε" morph="robinson:V-PAM-2P">Repent</w>: <w src="11" lemma="strong:G1063 lemma.TR:γαρ" morph="robinson:CONJ">for</w> <w src="12 13" lemma="strong:G3588 strong:G932 lemma.TR:η lemma.TR:βασιλεια" morph="robinson:T-NSF robinson:N-NSF">the kingdom</w> <w src="14 15" lemma="strong:G3588 strong:G3772 lemma.TR:των lemma.TR:ουρανων" morph="robinson:T-GPM robinson:N-GPM">of heaven</w> <w src="10" lemma="strong:G1448 lemma.TR:ηγγικεν" morph="robinson:V-RAI-3S">is at hand</w>.</q><verse eID="Matt.4.17"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Matt.4.17" sID="Matt.4.17"/><milestone type="x-p" subType="x-added" marker="¶"/><w>From</w> <w>that time</w> <w>Jesus</w> <w>began</w> <w>to preach</w>, <w>and</w> <w>to say</w>, <q who="Jesus" marker=""><w>Repent</w>: <w>for</w> <w>the kingdom</w> <w>of heaven</w> <w>is at hand</w>.</q><verse eID="Matt.4.17"/>
```

### MAT.4.19 (OSIS `Matt.4.19`)

- offset 15296206, length 1034, excerpt SHA-256 `c6bb20ab768fa129f383c15e87edde14261eee3eb6e0b7292c128dabb09c13d1`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 1; `</q>` end tags in excerpt: 1

```xml
<verse osisID="Matt.4.19" sID="Matt.4.19"/><w src="1" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">And</w> <w src="2" lemma="strong:G3004 lemma.TR:λεγει" morph="robinson:V-PAI-3S">he saith</w> <w src="3" lemma="strong:G846 lemma.TR:αυτοις" morph="robinson:P-DPM">unto them</w>, <q who="Jesus" marker=""><w src="4" lemma="strong:G1205 lemma.TR:δευτε" morph="robinson:V-PAM-2P">Follow</w> <w src="5" lemma="strong:G3694 lemma.TR:οπισω" morph="robinson:ADV"/><w src="6" lemma="strong:G1473 lemma.TR:μου" morph="robinson:P-1GS">me</w>, <w src="7" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="8" lemma="strong:G4160 lemma.TR:ποιησω" morph="robinson:V-FAI-1S">I will make</w> <w src="9" lemma="strong:G4771 lemma.TR:υμας" morph="robinson:P-2AP">you</w> <w src="10" lemma="strong:G231 lemma.TR:αλιεις" morph="robinson:N-APM">fishers</w> <w src="11" lemma="strong:G444 lemma.TR:ανθρωπων" morph="robinson:N-GPM">of men</w>.</q><verse eID="Matt.4.19"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Matt.4.19" sID="Matt.4.19"/><w>And</w> <w>he saith</w> <w>unto them</w>, <q who="Jesus" marker=""><w>Follow</w> <w/><w>me</w>, <w>and</w> <w>I will make</w> <w>you</w> <w>fishers</w> <w>of men</w>.</q><verse eID="Matt.4.19"/>
```

### MAT.5.2 (OSIS `Matt.5.2`)

- offset 15310214, length 727, excerpt SHA-256 `c7ec8878b56b9ddd1f894b687feadd49a34c83b3f2b9d997f51273a3a90542bf`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="Matt.5.2" sID="Matt.5.2"/><w src="1" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">And</w> <w src="2" lemma="strong:G455 lemma.TR:ανοιξας" morph="robinson:V-AAP-NSM">he opened</w> <w src="5" lemma="strong:G846 lemma.TR:αυτου" morph="robinson:P-GSM">his</w> <w src="3 4" lemma="strong:G3588 strong:G4750 lemma.TR:το lemma.TR:στομα" morph="robinson:T-ASN robinson:N-ASN">mouth</w>, <w src="6" lemma="strong:G1321 lemma.TR:εδιδασκεν" morph="robinson:V-IAI-3S">and taught</w> <w src="7" lemma="strong:G846 lemma.TR:αυτους" morph="robinson:P-APM">them</w>, <w src="8" lemma="strong:G3004 lemma.TR:λεγων" morph="robinson:V-PAP-NSM">saying</w>,<verse eID="Matt.5.2"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Matt.5.2" sID="Matt.5.2"/><w>And</w> <w>he opened</w> <w>his</w> <w>mouth</w>, <w>and taught</w> <w>them</w>, <w>saying</w>,<verse eID="Matt.5.2"/>
```

### MAT.5.3 (OSIS `Matt.5.3`)

- offset 15310942, length 1054, excerpt SHA-256 `d733ee5c51d5ca44c03e5ff2cd970cbff2ec779f2e2c83184aee812f05a2d1d0`
- `q` open at verse start: no
- `q` open at verse end: **yes** (continues into the next verse)
- `<q>` start tags in excerpt: 1; `</q>` end tags in excerpt: 0

```xml
<verse osisID="Matt.5.3" sID="Matt.5.3"/><q who="Jesus" marker=""><w src="1" lemma="strong:G3107 lemma.TR:μακαριοι" morph="robinson:A-NPM">Blessed</w> <transChange type="added">are</transChange> <w src="3" lemma="strong:G4434 lemma.TR:πτωχοι" morph="robinson:A-NPM">the poor</w> <w src="4 5" lemma="strong:G3588 strong:G4151 lemma.TR:τω lemma.TR:πνευματι" morph="robinson:T-DSN robinson:N-DSN">in spirit</w>: <w src="6" lemma="strong:G3754 lemma.TR:οτι" morph="robinson:CONJ">for</w> <w src="7" lemma="strong:G846 lemma.TR:αυτων" morph="robinson:P-GPM">theirs</w> <w src="8" lemma="strong:G1510 lemma.TR:εστιν" morph="robinson:V-PAI-3S">is</w> <w src="9 10" lemma="strong:G3588 strong:G932 lemma.TR:η lemma.TR:βασιλεια" morph="robinson:T-NSF robinson:N-NSF">the kingdom</w> <w src="11 12" lemma="strong:G3588 strong:G3772 lemma.TR:των lemma.TR:ουρανων" morph="robinson:T-GPM robinson:N-GPM">of heaven</w>.<w src="2" lemma="strong:G3588 lemma.TR:οι" morph="robinson:T-NPM"/><verse eID="Matt.5.3"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Matt.5.3" sID="Matt.5.3"/><q who="Jesus" marker=""><w>Blessed</w> <transChange type="added">are</transChange> <w>the poor</w> <w>in spirit</w>: <w>for</w> <w>theirs</w> <w>is</w> <w>the kingdom</w> <w>of heaven</w>.<w/><verse eID="Matt.5.3"/>
```

### MAT.5.12 (OSIS `Matt.5.12`)

- offset 15319795, length 1649, excerpt SHA-256 `38eb8797b94476315b9d0f76a48f9f7a4df328ea7aed62d8974ab5918baf3a10`
- `q` open at verse start: **yes**, its `<q>` tag is in `Matt.5.3` at byte 15310983
- `q` open at verse end: **yes** (continues into the next verse)
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="Matt.5.12" sID="Matt.5.12"/><w src="1" lemma="strong:G5463 lemma.TR:χαιρετε" morph="robinson:V-PAM-2P">Rejoice</w>, <w src="2" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="3" lemma="strong:G21 lemma.TR:αγαλλιασθε" morph="robinson:V-PNM-2P">be exceeding glad</w>: <w src="4" lemma="strong:G3754 lemma.TR:οτι" morph="robinson:CONJ">for</w> <w src="8" lemma="strong:G4183 lemma.TR:πολυς" morph="robinson:A-NSM">great</w> <transChange type="added">is</transChange> <w src="7" lemma="strong:G4771 lemma.TR:υμων" morph="robinson:P-2GP">your</w> <w src="5 6" lemma="strong:G3588 strong:G3408 lemma.TR:ο lemma.TR:μισθος" morph="robinson:T-NSM robinson:N-NSM">reward</w> <w src="9" lemma="strong:G1722 lemma.TR:εν" morph="robinson:PREP">in</w> <w src="10 11" lemma="strong:G3588 strong:G3772 lemma.TR:τοις lemma.TR:ουρανοις" morph="robinson:T-DPM robinson:N-DPM">heaven</w>: <w src="13" lemma="strong:G1063 lemma.TR:γαρ" morph="robinson:CONJ">for</w> <w src="12" lemma="strong:G3779 lemma.TR:ουτως" morph="robinson:ADV">so</w> <w src="14" lemma="strong:G1377 lemma.TR:εδιωξαν" morph="robinson:V-AAI-3P">persecuted they</w> <w src="15 16" lemma="strong:G3588 strong:G4396 lemma.TR:τους lemma.TR:προφητας" morph="robinson:T-APM robinson:N-APM">the prophets</w> <w src="17" lemma="strong:G3588 lemma.TR:τους" morph="robinson:T-APM">which were</w> <w src="18" lemma="strong:G4253 lemma.TR:προ" morph="robinson:PREP">before</w> <w src="19" lemma="strong:G4771 lemma.TR:υμων" morph="robinson:P-2GP">you</w>.<verse eID="Matt.5.12"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Matt.5.12" sID="Matt.5.12"/><w>Rejoice</w>, <w>and</w> <w>be exceeding glad</w>: <w>for</w> <w>great</w> <transChange type="added">is</transChange> <w>your</w> <w>reward</w> <w>in</w> <w>heaven</w>: <w>for</w> <w>so</w> <w>persecuted they</w> <w>the prophets</w> <w>which were</w> <w>before</w> <w>you</w>.<verse eID="Matt.5.12"/>
```

### MAT.5.13 (OSIS `Matt.5.13`)

- offset 15321445, length 2408, excerpt SHA-256 `8c453762217e682f8608926ab57534158bbc94a7fce01f6ed28c3d2940eda4e6`
- `q` open at verse start: **yes**, its `<q>` tag is in `Matt.5.3` at byte 15310983
- `q` open at verse end: **yes** (continues into the next verse)
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="Matt.5.13" sID="Matt.5.13"/><milestone type="x-p" marker="¶"/><w src="1" lemma="strong:G4771 lemma.TR:υμεις" morph="robinson:P-2NP">Ye</w> <w src="2" lemma="strong:G1510 lemma.TR:εστε" morph="robinson:V-PAI-2P">are</w> <w src="3 4" lemma="strong:G3588 strong:G217 lemma.TR:το lemma.TR:αλας" morph="robinson:T-NSN robinson:N-NSN">the salt</w> <w src="5 6" lemma="strong:G3588 strong:G1093 lemma.TR:της lemma.TR:γης" morph="robinson:T-GSF robinson:N-GSF">of the earth</w>: <w src="8" lemma="strong:G1161 lemma.TR:δε" morph="robinson:CONJ">but</w> <w src="7" lemma="strong:G1437 lemma.TR:εαν" morph="robinson:COND">if</w> <w src="9 10" lemma="strong:G3588 strong:G217 lemma.TR:το lemma.TR:αλας" morph="robinson:T-NSN robinson:N-NSN">the salt</w> <w src="11" lemma="strong:G3471 lemma.TR:μωρανθη" morph="robinson:V-APS-3S">have lost his savour</w>, <w src="12" lemma="strong:G1722 lemma.TR:εν" morph="robinson:PREP"/><w src="13" lemma="strong:G5101 lemma.TR:τινι" morph="robinson:I-DSN">wherewith</w> <w src="14" lemma="strong:G233 lemma.TR:αλισθησεται" morph="robinson:V-FPI-3S">shall it be salted</w>? <w src="17" lemma="strong:G2480 lemma.TR:ισχυει" morph="robinson:V-PAI-3S" type="x-split-420">it is</w> <w src="18" lemma="strong:G2089 lemma.TR:ετι" morph="robinson:ADV">thenceforth</w> <w src="17" lemma="strong:G2480 lemma.TR:ισχυει" morph="robinson:V-PAI-3S" type="x-split-420">good</w> <w src="15" lemma="strong:G1519 lemma.TR:εις" morph="robinson:PREP">for</w> <w src="16" lemma="strong:G3762 lemma.TR:ουδεν" morph="robinson:A-ASN-N">nothing</w>, <w src="19" lemma="strong:G1487 lemma.TR:ει" morph="robinson:COND"/><w src="20" lemma="strong:G3361 lemma.TR:μη" morph="robinson:PRT-N">but</w> <w src="21" lemma="strong:G906 lemma.TR:βληθηναι" morph="robinson:V-APN">to be cast</w> <w src="22" lemma="strong:G1854 lemma.TR:εξω" morph="robinson:ADV">out</w>, <w src="23" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="24" lemma="strong:G2662 lemma.TR:καταπατεισθαι" morph="robinson:V-PPN">to be trodden under foot</w> <w src="25" lemma="strong:G5259 lemma.TR:υπο" morph="robinson:PREP">of</w> <w src="26 27" lemma="strong:G3588 strong:G444 lemma.TR:των lemma.TR:ανθρωπων" morph="robinson:T-GPM robinson:N-GPM">men</w>.<verse eID="Matt.5.13"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Matt.5.13" sID="Matt.5.13"/><milestone type="x-p" marker="¶"/><w>Ye</w> <w>are</w> <w>the salt</w> <w>of the earth</w>: <w>but</w> <w>if</w> <w>the salt</w> <w>have lost his savour</w>, <w/><w>wherewith</w> <w>shall it be salted</w>? <w>it is</w> <w>thenceforth</w> <w>good</w> <w>for</w> <w>nothing</w>, <w/><w>but</w> <w>to be cast</w> <w>out</w>, <w>and</w> <w>to be trodden under foot</w> <w>of</w> <w>men</w>.<verse eID="Matt.5.13"/>
```

### MAT.5.14 (OSIS `Matt.5.14`)

- offset 15323854, length 1115, excerpt SHA-256 `b6e3157ebf43a0c2afaf5bcda58ff9c622c47005f61087086b6fd3511869d58b`
- `q` open at verse start: **yes**, its `<q>` tag is in `Matt.5.3` at byte 15310983
- `q` open at verse end: **yes** (continues into the next verse)
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="Matt.5.14" sID="Matt.5.14"/><w src="1" lemma="strong:G4771 lemma.TR:υμεις" morph="robinson:P-2NP">Ye</w> <w src="2" lemma="strong:G1510 lemma.TR:εστε" morph="robinson:V-PAI-2P">are</w> <w src="3 4" lemma="strong:G3588 strong:G5457 lemma.TR:το lemma.TR:φως" morph="robinson:T-NSN robinson:N-NSN">the light</w> <w src="5 6" lemma="strong:G3588 strong:G2889 lemma.TR:του lemma.TR:κοσμου" morph="robinson:T-GSM robinson:N-GSM">of the world</w>. <w src="9" lemma="strong:G4172 lemma.TR:πολις" morph="robinson:N-NSF">A city</w> <w src="13" lemma="strong:G2749 lemma.TR:κειμενη" morph="robinson:V-PNP-NSF">that is set</w> <w src="11" lemma="strong:G1883 lemma.TR:επανω" morph="robinson:ADV">on</w> <w src="12" lemma="strong:G3735 lemma.TR:ορους" morph="robinson:N-GSN">an hill</w> <w src="7" lemma="strong:G3756 lemma.TR:ου" morph="robinson:PRT-N"/><w src="8" lemma="strong:G1410 lemma.TR:δυναται" morph="robinson:V-PNI-3S">cannot</w> <w src="10" lemma="strong:G2928 lemma.TR:κρυβηναι" morph="robinson:V-2APN">be hid</w>.<verse eID="Matt.5.14"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Matt.5.14" sID="Matt.5.14"/><w>Ye</w> <w>are</w> <w>the light</w> <w>of the world</w>. <w>A city</w> <w>that is set</w> <w>on</w> <w>an hill</w> <w/><w>cannot</w> <w>be hid</w>.<verse eID="Matt.5.14"/>
```

### MAT.9.6 (OSIS `Matt.9.6`)

- offset 15555642, length 2481, excerpt SHA-256 `915ad8e54a6405d536ee46a774181b0b3991aefd0864374a22ce378194d98702`
- `q` open at verse start: **yes**, its `<q>` tag is in `Matt.9.4` at byte 15553578
- `q` open at verse end: no
- `<q>` start tags in excerpt: 1; `</q>` end tags in excerpt: 2

```xml
<verse osisID="Matt.9.6" sID="Matt.9.6"/><w src="18" lemma="strong:G3588 lemma.TR:τω" morph="robinson:T-DSM"/><w src="2" lemma="strong:G1161 lemma.TR:δε" morph="robinson:CONJ">But</w> <w src="1" lemma="strong:G2443 lemma.TR:ινα" morph="robinson:CONJ">that</w> <w src="3" lemma="strong:G1492 lemma.TR:ειδητε" morph="robinson:V-RAS-2P">ye may know</w> <w src="4" lemma="strong:G3754 lemma.TR:οτι" morph="robinson:CONJ">that</w> <w src="7 8" lemma="strong:G3588 strong:G5207 lemma.TR:ο lemma.TR:υιος" morph="robinson:T-NSM robinson:N-NSM">the Son</w> <w src="9 10" lemma="strong:G3588 strong:G444 lemma.TR:του lemma.TR:ανθρωπου" morph="robinson:T-GSM robinson:N-GSM">of man</w> <w src="6" lemma="strong:G2192 lemma.TR:εχει" morph="robinson:V-PAI-3S">hath</w> <w src="5" lemma="strong:G1849 lemma.TR:εξουσιαν" morph="robinson:N-ASF">power</w> <w src="11" lemma="strong:G1909 lemma.TR:επι" morph="robinson:PREP">on</w> <w src="12 13" lemma="strong:G3588 strong:G1093 lemma.TR:της lemma.TR:γης" morph="robinson:T-GSF robinson:N-GSF">earth</w> <w src="14" lemma="strong:G863 lemma.TR:αφιεναι" morph="robinson:V-PAN">to forgive</w> <w src="15" lemma="strong:G266 lemma.TR:αμαρτιας" morph="robinson:N-APF">sins</w>,</q> (<w src="16" lemma="strong:G5119 lemma.TR:τοτε" morph="robinson:ADV">then</w> <w src="17" lemma="strong:G3004 lemma.TR:λεγει" morph="robinson:V-PAI-3S">saith he</w> <w src="19" lemma="strong:G3885 lemma.TR:παραλυτικω" morph="robinson:A-DSM">to the sick of the palsy</w>,) <q who="Jesus" marker=""><w src="20" lemma="strong:G1453 lemma.TR:εγερθεις" morph="robinson:V-APP-NSM">Arise</w>, <w src="21" lemma="strong:G142 lemma.TR:αρον" morph="robinson:V-AAM-2S">take up</w> <w src="22" lemma="strong:G4771 lemma.TR:σου" morph="robinson:P-2GS">thy</w> <w src="23 24" lemma="strong:G3588 strong:G2825 lemma.TR:την lemma.TR:κλινην" morph="robinson:T-ASF robinson:N-ASF">bed</w>, <w src="25" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="26" lemma="strong:G5217 lemma.TR:υπαγε" morph="robinson:V-PAM-2S">go</w> <w src="27" lemma="strong:G1519 lemma.TR:εις" morph="robinson:PREP">unto</w> <w src="30" lemma="strong:G4771 lemma.TR:σου" morph="robinson:P-2GS">thine</w> <w src="28 29" lemma="strong:G3588 strong:G3624 lemma.TR:τον lemma.TR:οικον" morph="robinson:T-ASM robinson:N-ASM">house</w>.</q><verse eID="Matt.9.6"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Matt.9.6" sID="Matt.9.6"/><w/><w>But</w> <w>that</w> <w>ye may know</w> <w>that</w> <w>the Son</w> <w>of man</w> <w>hath</w> <w>power</w> <w>on</w> <w>earth</w> <w>to forgive</w> <w>sins</w>,</q> (<w>then</w> <w>saith he</w> <w>to the sick of the palsy</w>,) <q who="Jesus" marker=""><w>Arise</w>, <w>take up</w> <w>thy</w> <w>bed</w>, <w>and</w> <w>go</w> <w>unto</w> <w>thine</w> <w>house</w>.</q><verse eID="Matt.9.6"/>
```

### MAT.22.21 (OSIS `Matt.22.21`)

- offset 16398324, length 1533, excerpt SHA-256 `0ac67e529eef6d2bf43c186652b70220d34885e058f02685fa39ed2921f2db06`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 1; `</q>` end tags in excerpt: 1

```xml
<verse osisID="Matt.22.21" sID="Matt.22.21"/><w src="13" lemma="strong:G3588 lemma.TR:τα" morph="robinson:T-APN"/><w src="9" lemma="strong:G3588 lemma.TR:τα" morph="robinson:T-APN"/><w src="1" lemma="strong:G3004 lemma.TR:λεγουσιν" morph="robinson:V-PAI-3P">They say</w> <w src="2" lemma="strong:G846 lemma.TR:αυτω" morph="robinson:P-DSM">unto him</w>, <w src="3" lemma="strong:G2541 lemma.TR:καισαρος" morph="robinson:N-GSM">Cesar’s</w>. <w src="4" lemma="strong:G5119 lemma.TR:τοτε" morph="robinson:ADV">Then</w> <w src="5" lemma="strong:G3004 lemma.TR:λεγει" morph="robinson:V-PAI-3S">saith he</w> <w src="6" lemma="strong:G846 lemma.TR:αυτοις" morph="robinson:P-DPM">unto them</w>, <q who="Jesus" marker=""><w src="7" lemma="strong:G591 lemma.TR:αποδοτε" morph="robinson:V-2AAM-2P">Render</w> <w src="8" lemma="strong:G3767 lemma.TR:ουν" morph="robinson:CONJ">therefore</w> <w src="11" lemma="strong:G2541 lemma.TR:καισαρι" morph="robinson:N-DSM">unto Cesar</w> <w src="10" lemma="strong:G2541 lemma.TR:καισαρος" morph="robinson:N-GSM">the things which are Cesar’s</w>; <w src="12" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="16 17" lemma="strong:G3588 strong:G2316 lemma.TR:τω lemma.TR:θεω" morph="robinson:T-DSM robinson:N-DSM">unto God</w> <w src="14 15" lemma="strong:G3588 strong:G2316 lemma.TR:του lemma.TR:θεου" morph="robinson:T-GSM robinson:N-GSM">the things that are God’s</w>.</q><verse eID="Matt.22.21"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Matt.22.21" sID="Matt.22.21"/><w/><w/><w>They say</w> <w>unto him</w>, <w>Cesar’s</w>. <w>Then</w> <w>saith he</w> <w>unto them</w>, <q who="Jesus" marker=""><w>Render</w> <w>therefore</w> <w>unto Cesar</w> <w>the things which are Cesar’s</w>; <w>and</w> <w>unto God</w> <w>the things that are God’s</w>.</q><verse eID="Matt.22.21"/>
```

### MAT.22.24 (OSIS `Matt.22.24`)

- offset 16402172, length 2135, excerpt SHA-256 `93822742d1acd2c0a9c18c8ff96c1fa4118501fd47b4cd646a0309b8c7f0bed5`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="Matt.22.24" sID="Matt.22.24"/><w src="1" lemma="strong:G3004 lemma.TR:λεγοντες" morph="robinson:V-PAP-NPM">Saying</w>, <w src="2" lemma="strong:G1320 lemma.TR:διδασκαλε" morph="robinson:N-VSM">Master</w>, <w src="3" lemma="strong:G3475 lemma.TR:μωσης" morph="robinson:N-NSM">Moses</w> <w src="4" lemma="strong:G3004 lemma.TR:ειπεν" morph="robinson:V-2AAI-3S">said</w>, <w src="5" lemma="strong:G1437 lemma.TR:εαν" morph="robinson:COND">If</w> <w src="6" lemma="strong:G5100 lemma.TR:τις" morph="robinson:X-NSM">a man</w> <w src="7" lemma="strong:G599 lemma.TR:αποθανη" morph="robinson:V-2AAS-3S">die</w>, <w src="9" lemma="strong:G2192 lemma.TR:εχων" morph="robinson:V-PAP-NSM">having</w> <w src="8" lemma="strong:G3361 lemma.TR:μη" morph="robinson:PRT-N">no</w> <w src="10" lemma="strong:G5043 lemma.TR:τεκνα" morph="robinson:N-APN">children</w>, <w src="14" lemma="strong:G846 lemma.TR:αυτου" morph="robinson:P-GSM">his</w> <w src="12 13" lemma="strong:G3588 strong:G80 lemma.TR:ο lemma.TR:αδελφος" morph="robinson:T-NSM robinson:N-NSM">brother</w> <w src="11" lemma="strong:G1918 lemma.TR:επιγαμβρευσει" morph="robinson:V-FAI-3S">shall marry</w> <w src="17" lemma="strong:G846 lemma.TR:αυτου" morph="robinson:P-GSM">his</w> <w src="15 16" lemma="strong:G3588 strong:G1135 lemma.TR:την lemma.TR:γυναικα" morph="robinson:T-ASF robinson:N-ASF">wife</w>, <w src="18" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="19" lemma="strong:G450 lemma.TR:αναστησει" morph="robinson:V-FAI-3S">raise up</w> <w src="20" lemma="strong:G4690 lemma.TR:σπερμα" morph="robinson:N-ASN">seed</w> <w src="21 22" lemma="strong:G3588 strong:G80 lemma.TR:τω lemma.TR:αδελφω" morph="robinson:T-DSM robinson:N-DSM" type="x-split-819">unto</w> <w src="23" lemma="strong:G846 lemma.TR:αυτου" morph="robinson:P-GSM">his</w> <w src="21 22" lemma="strong:G3588 strong:G80 lemma.TR:τω lemma.TR:αδελφω" morph="robinson:T-DSM robinson:N-DSM" type="x-split-819">brother</w>.<verse eID="Matt.22.24"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Matt.22.24" sID="Matt.22.24"/><w>Saying</w>, <w>Master</w>, <w>Moses</w> <w>said</w>, <w>If</w> <w>a man</w> <w>die</w>, <w>having</w> <w>no</w> <w>children</w>, <w>his</w> <w>brother</w> <w>shall marry</w> <w>his</w> <w>wife</w>, <w>and</w> <w>raise up</w> <w>seed</w> <w>unto</w> <w>his</w> <w>brother</w>.<verse eID="Matt.22.24"/>
```

### MAT.26.26 (OSIS `Matt.26.26`)

- offset 16682668, length 2336, excerpt SHA-256 `3af086bc7c9d05cb4fb92a84df0fe976753f1b446c1172a95efbe6c0e50bf3fa`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 1; `</q>` end tags in excerpt: 1

```xml
<verse osisID="Matt.26.26" sID="Matt.26.26"/><milestone type="x-p" marker="¶"/><w src="2" lemma="strong:G1161 lemma.TR:δε" morph="robinson:CONJ">And</w> <w src="1" lemma="strong:G2068 lemma.TR:εσθιοντων" morph="robinson:V-PAP-GPM" type="x-split-944">as</w> <w src="3" lemma="strong:G846 lemma.TR:αυτων" morph="robinson:P-GPM">they</w> <w src="1" lemma="strong:G2068 lemma.TR:εσθιοντων" morph="robinson:V-PAP-GPM" type="x-split-944">were eating</w>, <w src="5 6" lemma="strong:G3588 strong:G2424 lemma.TR:ο lemma.TR:ιησους" morph="robinson:T-NSM robinson:N-NSM">Jesus</w> <w src="4" lemma="strong:G2983 lemma.TR:λαβων" morph="robinson:V-2AAP-NSM">took</w> <w src="7 8" lemma="strong:G3588 strong:G740 lemma.TR:τον lemma.TR:αρτον" morph="robinson:T-ASM robinson:N-ASM">bread</w>, <w src="9" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="10" lemma="strong:G2127 lemma.TR:ευλογησας" morph="robinson:V-AAP-NSM">blessed</w> <transChange type="added">it</transChange>, <w src="11" lemma="strong:G2806 lemma.TR:εκλασεν" morph="robinson:V-AAI-3S">and brake</w> <transChange type="added">it</transChange>, <w src="12" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="13" lemma="strong:G1325 lemma.TR:εδιδου" morph="robinson:V-IAI-3S">gave</w> <transChange type="added">it</transChange> <w src="14 15" lemma="strong:G3588 strong:G3101 lemma.TR:τοις lemma.TR:μαθηταις" morph="robinson:T-DPM robinson:N-DPM">to the disciples</w>, <w src="16" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="17" lemma="strong:G3004 lemma.TR:ειπεν" morph="robinson:V-2AAI-3S">said</w>, <q who="Jesus" marker=""><w src="18" lemma="strong:G2983 lemma.TR:λαβετε" morph="robinson:V-2AAM-2P">Take</w>, <w src="19" lemma="strong:G5315 lemma.TR:φαγετε" morph="robinson:V-2AAM-2P">eat</w>; <w src="20" lemma="strong:G3778 lemma.TR:τουτο" morph="robinson:D-NSN">this</w> <w src="21" lemma="strong:G1510 lemma.TR:εστιν" morph="robinson:V-PAI-3S">is</w> <w src="24" lemma="strong:G1473 lemma.TR:μου" morph="robinson:P-1GS">my</w> <w src="22 23" lemma="strong:G3588 strong:G4983 lemma.TR:το lemma.TR:σωμα" morph="robinson:T-NSN robinson:N-NSN">body</w>.</q><verse eID="Matt.26.26"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Matt.26.26" sID="Matt.26.26"/><milestone type="x-p" marker="¶"/><w>And</w> <w>as</w> <w>they</w> <w>were eating</w>, <w>Jesus</w> <w>took</w> <w>bread</w>, <w>and</w> <w>blessed</w> <transChange type="added">it</transChange>, <w>and brake</w> <transChange type="added">it</transChange>, <w>and</w> <w>gave</w> <transChange type="added">it</transChange> <w>to the disciples</w>, <w>and</w> <w>said</w>, <q who="Jesus" marker=""><w>Take</w>, <w>eat</w>; <w>this</w> <w>is</w> <w>my</w> <w>body</w>.</q><verse eID="Matt.26.26"/>
```

### MAT.26.27 (OSIS `Matt.26.27`)

- offset 16685005, length 1244, excerpt SHA-256 `4248578f4ad4a3cd216a86fd17342861465568ce41e9451ffaf5c1649cc722f6`
- `q` open at verse start: no
- `q` open at verse end: **yes** (continues into the next verse)
- `<q>` start tags in excerpt: 1; `</q>` end tags in excerpt: 0

```xml
<verse osisID="Matt.26.27" sID="Matt.26.27"/><w src="1" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">And</w> <w src="2" lemma="strong:G2983 lemma.TR:λαβων" morph="robinson:V-2AAP-NSM">he took</w> <w src="3 4" lemma="strong:G3588 strong:G4221 lemma.TR:το lemma.TR:ποτηριον" morph="robinson:T-ASN robinson:N-ASN">the cup</w>, <w src="5" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="6" lemma="strong:G2168 lemma.TR:ευχαριστησας" morph="robinson:V-AAP-NSM">gave thanks</w>, <w src="7" lemma="strong:G1325 lemma.TR:εδωκεν" morph="robinson:V-AAI-3S">and gave</w> <transChange type="added">it</transChange> <w src="8" lemma="strong:G846 lemma.TR:αυτοις" morph="robinson:P-DPM">to them</w>, <w src="9" lemma="strong:G3004 lemma.TR:λεγων" morph="robinson:V-PAP-NSM">saying</w>, <q who="Jesus" marker=""><w src="10" lemma="strong:G4095 lemma.TR:πιετε" morph="robinson:V-2AAM-2P">Drink ye</w> <w src="13" lemma="strong:G3956 lemma.TR:παντες" morph="robinson:A-NPM">all</w> <w src="11" lemma="strong:G1537 lemma.TR:εξ" morph="robinson:PREP">of</w> <w src="12" lemma="strong:G846 lemma.TR:αυτου" morph="robinson:P-GSN">it</w>;<verse eID="Matt.26.27"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Matt.26.27" sID="Matt.26.27"/><w>And</w> <w>he took</w> <w>the cup</w>, <w>and</w> <w>gave thanks</w>, <w>and gave</w> <transChange type="added">it</transChange> <w>to them</w>, <w>saying</w>, <q who="Jesus" marker=""><w>Drink ye</w> <w>all</w> <w>of</w> <w>it</w>;<verse eID="Matt.26.27"/>
```

### MAT.26.28 (OSIS `Matt.26.28`)

- offset 16686250, length 1592, excerpt SHA-256 `6e9c34e52c6c931be6b463db902d4d36e45c1c994c061bd57b11b96a3afcfc02`
- `q` open at verse start: **yes**, its `<q>` tag is in `Matt.26.27` at byte 16685862
- `q` open at verse end: **yes** (continues into the next verse)
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="Matt.26.28" sID="Matt.26.28"/><w src="11" lemma="strong:G3588 lemma.TR:το" morph="robinson:T-NSN"/><w src="8" lemma="strong:G3588 lemma.TR:της" morph="robinson:T-GSF"/><w src="7" lemma="strong:G3588 lemma.TR:το" morph="robinson:T-NSN"/><w src="2" lemma="strong:G1063 lemma.TR:γαρ" morph="robinson:CONJ">For</w> <w src="1" lemma="strong:G3778 lemma.TR:τουτο" morph="robinson:D-NSN">this</w> <w src="3" lemma="strong:G1510 lemma.TR:εστιν" morph="robinson:V-PAI-3S">is</w> <w src="6" lemma="strong:G1473 lemma.TR:μου" morph="robinson:P-1GS">my</w> <w src="4 5" lemma="strong:G3588 strong:G129 lemma.TR:το lemma.TR:αιμα" morph="robinson:T-NSN robinson:N-NSN">blood</w> <w src="10" lemma="strong:G1242 lemma.TR:διαθηκης" morph="robinson:N-GSF" type="x-split-945">of the</w> <w src="9" lemma="strong:G2537 lemma.TR:καινης" morph="robinson:A-GSF">new</w> <w src="10" lemma="strong:G1242 lemma.TR:διαθηκης" morph="robinson:N-GSF" type="x-split-945">testament</w>, <w src="14" lemma="strong:G1632 lemma.TR:εκχυνομενον" morph="robinson:V-PPP-NSN">which is shed</w> <w src="12" lemma="strong:G4012 lemma.TR:περι" morph="robinson:PREP">for</w> <w src="13" lemma="strong:G4183 lemma.TR:πολλων" morph="robinson:A-GPM">many</w> <w src="15" lemma="strong:G1519 lemma.TR:εις" morph="robinson:PREP">for</w> <w src="16" lemma="strong:G859 lemma.TR:αφεσιν" morph="robinson:N-ASF">the remission</w> <w src="17" lemma="strong:G266 lemma.TR:αμαρτιων" morph="robinson:N-GPF">of sins</w>.<verse eID="Matt.26.28"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Matt.26.28" sID="Matt.26.28"/><w/><w/><w/><w>For</w> <w>this</w> <w>is</w> <w>my</w> <w>blood</w> <w>of the</w> <w>new</w> <w>testament</w>, <w>which is shed</w> <w>for</w> <w>many</w> <w>for</w> <w>the remission</w> <w>of sins</w>.<verse eID="Matt.26.28"/>
```

### MAT.27.46 (OSIS `Matt.27.46`)

- offset 16828382, length 2399, excerpt SHA-256 `05acb36197e01275d43c9c6277a2e5ea8d85ce8e4c19206711091f89bf989e6a`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 1; `</q>` end tags in excerpt: 1

```xml
<verse osisID="Matt.27.46" sID="Matt.27.46"/><w src="2" lemma="strong:G1161 lemma.TR:δε" morph="robinson:CONJ">And</w> <w src="1" lemma="strong:G4012 lemma.TR:περι" morph="robinson:PREP">about</w> <w src="3 5" lemma="strong:G3588 strong:G5610 lemma.TR:την lemma.TR:ωραν" morph="robinson:T-ASF robinson:N-ASF" type="x-split-979">the</w> <w src="4" lemma="strong:G1766 lemma.TR:εννατην" morph="robinson:A-ASF">ninth</w> <w src="3 5" lemma="strong:G3588 strong:G5610 lemma.TR:την lemma.TR:ωραν" morph="robinson:T-ASF robinson:N-ASF" type="x-split-979">hour</w> <w src="7 8" lemma="strong:G3588 strong:G2424 lemma.TR:ο lemma.TR:ιησους" morph="robinson:T-NSM robinson:N-NSM">Jesus</w> <w src="6" lemma="strong:G310 lemma.TR:ανεβοησεν" morph="robinson:V-AAI-3S">cried with</w> <w src="9" lemma="strong:G5456 lemma.TR:φωνη" morph="robinson:N-DSF" type="x-split-980">a</w> <w src="10" lemma="strong:G3173 lemma.TR:μεγαλη" morph="robinson:A-DSF">loud</w> <w src="9" lemma="strong:G5456 lemma.TR:φωνη" morph="robinson:N-DSF" type="x-split-980">voice</w>, <w src="11" lemma="strong:G3004 lemma.TR:λεγων" morph="robinson:V-PAP-NSM">saying</w>, <q who="Jesus" marker=""><w src="12" lemma="strong:G2241 lemma.TR:ηλι" morph="robinson:HEB">Eli</w>, <w src="13" lemma="strong:G2241 lemma.TR:ηλι" morph="robinson:HEB">Eli</w>, <w src="14" lemma="strong:G2982 lemma.TR:λαμα" morph="robinson:HEB">lama</w> <w src="15" lemma="strong:G4518 lemma.TR:σαβαχθανι" morph="robinson:HEB">sabachthani</w>?</q> <w src="16" lemma="strong:G3778 lemma.TR:τουτ" morph="robinson:D-NSN">that</w> <w src="17" lemma="strong:G1510 lemma.TR:εστιν" morph="robinson:V-PAI-3S">is to say</w>, <w src="19" lemma="strong:G1473 lemma.TR:μου" morph="robinson:P-1GS">My</w> <w src="18" lemma="strong:G2316 lemma.TR:θεε" morph="robinson:N-VSM">God</w>, <w src="21" lemma="strong:G1473 lemma.TR:μου" morph="robinson:P-1GS">my</w> <w src="20" lemma="strong:G2316 lemma.TR:θεε" morph="robinson:N-VSM">God</w>, <w src="22 23" lemma="strong:G2443 strong:G5101 lemma.TR:ινα lemma.TR:τι" morph="robinson:CONJ robinson:I-NSN">why</w> <w src="25" lemma="strong:G1459 lemma.TR:εγκατελιπες" morph="robinson:V-2AAI-2S">hast thou forsaken</w> <w src="24" lemma="strong:G1473 lemma.TR:με" morph="robinson:P-1AS">me</w>?<verse eID="Matt.27.46"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Matt.27.46" sID="Matt.27.46"/><w>And</w> <w>about</w> <w>the</w> <w>ninth</w> <w>hour</w> <w>Jesus</w> <w>cried with</w> <w>a</w> <w>loud</w> <w>voice</w>, <w>saying</w>, <q who="Jesus" marker=""><w>Eli</w>, <w>Eli</w>, <w>lama</w> <w>sabachthani</w>?</q> <w>that</w> <w>is to say</w>, <w>My</w> <w>God</w>, <w>my</w> <w>God</w>, <w>why</w> <w>hast thou forsaken</w> <w>me</w>?<verse eID="Matt.27.46"/>
```

### MRK.5.41 (OSIS `Mark.5.41`)

- offset 17182205, length 1597, excerpt SHA-256 `35a9d8248856d6849d115e1cc638ea4a7e1c94080906c9dea1c1b7a02fae564b`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 1; `</q>` end tags in excerpt: 1

```xml
<verse osisID="Mark.5.41" sID="Mark.5.41"/><w src="1" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">And</w> <w src="2" lemma="strong:G2902 lemma.TR:κρατησας" morph="robinson:V-AAP-NSM">he took</w> <w src="5 6" lemma="strong:G3588 strong:G3813 lemma.TR:του lemma.TR:παιδιου" morph="robinson:T-GSN robinson:N-GSN">the damsel</w> <w src="3 4" lemma="strong:G3588 strong:G5495 lemma.TR:της lemma.TR:χειρος" morph="robinson:T-GSF robinson:N-GSF">by the hand</w>, <w src="7" lemma="strong:G3004 lemma.TR:λεγει" morph="robinson:V-PAI-3S">and said</w> <w src="8" lemma="strong:G846 lemma.TR:αυτη" morph="robinson:P-DSF">unto her</w>, <q who="Jesus" marker=""><w src="9" lemma="strong:G5008 lemma.TR:ταλιθα" morph="robinson:ARAM">Talitha</w> <w src="10" lemma="strong:G2891 lemma.TR:κουμι" morph="robinson:ARAM">cumi</w>;</q> <w src="11" lemma="strong:G3739 lemma.TR:ο" morph="robinson:R-NSN">which</w> <w src="12" lemma="strong:G1510 lemma.TR:εστιν" morph="robinson:V-PAI-3S">is</w>, <w src="13" lemma="strong:G3177 lemma.TR:μεθερμηνευομενον" morph="robinson:V-PPP-NSN">being interpreted</w>, <w src="14 15" lemma="strong:G3588 strong:G2877 lemma.TR:το lemma.TR:κορασιον" morph="robinson:T-NSN robinson:N-NSN">Damsel</w>, <w src="17" lemma="strong:G3004 lemma.TR:λεγω" morph="robinson:V-PAI-1S">I say</w> <w src="16" lemma="strong:G4771 lemma.TR:σοι" morph="robinson:P-2DS">unto thee</w>, <w src="18" lemma="strong:G1453 lemma.TR:εγειραι" morph="robinson:V-AMM-2S">arise</w>.<verse eID="Mark.5.41"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Mark.5.41" sID="Mark.5.41"/><w>And</w> <w>he took</w> <w>the damsel</w> <w>by the hand</w>, <w>and said</w> <w>unto her</w>, <q who="Jesus" marker=""><w>Talitha</w> <w>cumi</w>;</q> <w>which</w> <w>is</w>, <w>being interpreted</w>, <w>Damsel</w>, <w>I say</w> <w>unto thee</w>, <w>arise</w>.<verse eID="Mark.5.41"/>
```

### MRK.7.34 (OSIS `Mark.7.34`)

- offset 17330071, length 1203, excerpt SHA-256 `68c3cf976057c06f5e4ba4759918966165951a05c86540a5966315d1f8606ce1`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 1; `</q>` end tags in excerpt: 1

```xml
<verse osisID="Mark.7.34" sID="Mark.7.34"/><w src="1" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">And</w> <w src="2" lemma="strong:G308 lemma.TR:αναβλεψας" morph="robinson:V-AAP-NSM">looking up</w> <w src="3" lemma="strong:G1519 lemma.TR:εις" morph="robinson:PREP">to</w> <w src="4 5" lemma="strong:G3588 strong:G3772 lemma.TR:τον lemma.TR:ουρανον" morph="robinson:T-ASM robinson:N-ASM">heaven</w>, <w src="6" lemma="strong:G4727 lemma.TR:εστεναξεν" morph="robinson:V-AAI-3S">he sighed</w>, <w src="7" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="8" lemma="strong:G3004 lemma.TR:λεγει" morph="robinson:V-PAI-3S">saith</w> <w src="9" lemma="strong:G846 lemma.TR:αυτω" morph="robinson:P-DSM">unto him</w>, <q who="Jesus" marker=""><w src="10" lemma="strong:G2188 lemma.TR:εφφαθα" morph="robinson:ARAM">Ephphatha</w>,</q> <w src="11" lemma="strong:G3739 lemma.TR:ο" morph="robinson:R-NSN">that</w> <w src="12" lemma="strong:G1510 lemma.TR:εστιν" morph="robinson:V-PAI-3S">is</w>, <w src="13" lemma="strong:G1272 lemma.TR:διανοιχθητι" morph="robinson:V-APM-2S">Be opened</w>.<verse eID="Mark.7.34"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Mark.7.34" sID="Mark.7.34"/><w>And</w> <w>looking up</w> <w>to</w> <w>heaven</w>, <w>he sighed</w>, <w>and</w> <w>saith</w> <w>unto him</w>, <q who="Jesus" marker=""><w>Ephphatha</w>,</q> <w>that</w> <w>is</w>, <w>Be opened</w>.<verse eID="Mark.7.34"/>
```

### MRK.8.1 (OSIS `Mark.8.1`)

- offset 17335932, length 1744, excerpt SHA-256 `fd64a31e3964454de821e582f208d2e3f429b71e9cf74c06a9107536cd0522ce`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="Mark.8.1" sID="Mark.8.1"/><w src="1" lemma="strong:G1722 lemma.TR:εν" morph="robinson:PREP">In</w> <w src="2" lemma="strong:G1565 lemma.TR:εκειναις" morph="robinson:D-DPF">those</w> <w src="3 4" lemma="strong:G3588 strong:G2250 lemma.TR:ταις lemma.TR:ημεραις" morph="robinson:T-DPF robinson:N-DPF">days</w> <w src="6" lemma="strong:G3793 lemma.TR:οχλου" morph="robinson:N-GSM">the multitude</w> <w src="7" lemma="strong:G1510 lemma.TR:οντος" morph="robinson:V-PAP-GSM">being</w> <w src="5" lemma="strong:G3827 lemma.TR:παμπολλου" morph="robinson:A-GSM">very great</w>, <w src="8" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="10" lemma="strong:G2192 lemma.TR:εχοντων" morph="robinson:V-PAP-GPM">having</w> <w src="11 9" lemma="strong:G5101 strong:G3361 lemma.TR:τι lemma.TR:μη" morph="robinson:I-ASN robinson:PRT-N">nothing</w> <w src="12" lemma="strong:G5315 lemma.TR:φαγωσιν" morph="robinson:V-2AAS-3P">to eat</w>, <w src="14 15" lemma="strong:G3588 strong:G2424 lemma.TR:ο lemma.TR:ιησους" morph="robinson:T-NSM robinson:N-NSM">Jesus</w> <w src="13" lemma="strong:G4341 lemma.TR:προσκαλεσαμενος" morph="robinson:V-ADP-NSM">called</w> <w src="18" lemma="strong:G846 lemma.TR:αυτου" morph="robinson:P-GSM">his</w> <w src="16 17" lemma="strong:G3588 strong:G3101 lemma.TR:τους lemma.TR:μαθητας" morph="robinson:T-APM robinson:N-APM">disciples</w> <transChange type="added">unto him</transChange>, <w src="19" lemma="strong:G3004 lemma.TR:λεγει" morph="robinson:V-PAI-3S">and saith</w> <w src="20" lemma="strong:G846 lemma.TR:αυτοις" morph="robinson:P-DPM">unto them</w>,<verse eID="Mark.8.1"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Mark.8.1" sID="Mark.8.1"/><w>In</w> <w>those</w> <w>days</w> <w>the multitude</w> <w>being</w> <w>very great</w>, <w>and</w> <w>having</w> <w>nothing</w> <w>to eat</w>, <w>Jesus</w> <w>called</w> <w>his</w> <w>disciples</w> <transChange type="added">unto him</transChange>, <w>and saith</w> <w>unto them</w>,<verse eID="Mark.8.1"/>
```

### MRK.8.2 (OSIS `Mark.8.2`)

- offset 17337677, length 1479, excerpt SHA-256 `8e52aa5e798f3862f99c7fd9195f231fee471a1d4b42cef8bee5e104454e0500`
- `q` open at verse start: no
- `q` open at verse end: **yes** (continues into the next verse)
- `<q>` start tags in excerpt: 1; `</q>` end tags in excerpt: 0

```xml
<verse osisID="Mark.8.2" sID="Mark.8.2"/><q who="Jesus" marker=""><w src="1" lemma="strong:G4697 lemma.TR:σπλαγχνιζομαι" morph="robinson:V-PNI-1S">I have compassion</w> <w src="2" lemma="strong:G1909 lemma.TR:επι" morph="robinson:PREP">on</w> <w src="3 4" lemma="strong:G3588 strong:G3793 lemma.TR:τον lemma.TR:οχλον" morph="robinson:T-ASM robinson:N-ASM">the multitude</w>, <w src="5" lemma="strong:G3754 lemma.TR:οτι" morph="robinson:CONJ">because</w> <w src="9" lemma="strong:G4357 lemma.TR:προσμενουσιν" morph="robinson:V-PAI-3P" type="x-split-1108">they have</w> <w src="6" lemma="strong:G2235 lemma.TR:ηδη" morph="robinson:ADV">now</w> <w src="9" lemma="strong:G4357 lemma.TR:προσμενουσιν" morph="robinson:V-PAI-3P" type="x-split-1108">been with</w> <w src="10" lemma="strong:G1473 lemma.TR:μοι" morph="robinson:P-1DS">me</w> <w src="8" lemma="strong:G5140 lemma.TR:τρεις" morph="robinson:A-APF">three</w> <w src="7" lemma="strong:G2250 lemma.TR:ημερας" morph="robinson:N-APF">days</w>, <w src="11" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="13" lemma="strong:G2192 lemma.TR:εχουσιν" morph="robinson:V-PAI-3P">have</w> <w src="12 14" lemma="strong:G3756 strong:G5101 lemma.TR:ουκ lemma.TR:τι" morph="robinson:PRT-N robinson:I-ASN">nothing</w> <w src="15" lemma="strong:G5315 lemma.TR:φαγωσιν" morph="robinson:V-2AAS-3P">to eat</w>:<verse eID="Mark.8.2"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Mark.8.2" sID="Mark.8.2"/><q who="Jesus" marker=""><w>I have compassion</w> <w>on</w> <w>the multitude</w>, <w>because</w> <w>they have</w> <w>now</w> <w>been with</w> <w>me</w> <w>three</w> <w>days</w>, <w>and</w> <w>have</w> <w>nothing</w> <w>to eat</w>:<verse eID="Mark.8.2"/>
```

### MRK.15.34 (OSIS `Mark.15.34`)

- offset 17906931, length 2321, excerpt SHA-256 `f03990c29ff2114e054a0e081556ce8dcd6c0e53d39801443531c019d5b14003`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 1; `</q>` end tags in excerpt: 1

```xml
<verse osisID="Mark.15.34" sID="Mark.15.34"/><w src="1" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">And</w> <w src="4 5" lemma="strong:G3588 strong:G1766 lemma.TR:τη lemma.TR:εννατη" morph="robinson:T-DSF robinson:A-DSF">at the ninth</w> <w src="2 3" lemma="strong:G3588 strong:G5610 lemma.TR:τη lemma.TR:ωρα" morph="robinson:T-DSF robinson:N-DSF">hour</w> <w src="7 8" lemma="strong:G3588 strong:G2424 lemma.TR:ο lemma.TR:ιησους" morph="robinson:T-NSM robinson:N-NSM">Jesus</w> <w src="6" lemma="strong:G994 lemma.TR:εβοησεν" morph="robinson:V-AAI-3S">cried</w> <w src="10" lemma="strong:G3173 lemma.TR:μεγαλη" morph="robinson:A-DSF">with a loud</w> <w src="9" lemma="strong:G5456 lemma.TR:φωνη" morph="robinson:N-DSF">voice</w>, <w src="11" lemma="strong:G3004 lemma.TR:λεγων" morph="robinson:V-PAP-NSM">saying</w>, <q who="Jesus" marker=""><w src="12" lemma="strong:G1682 lemma.TR:ελωι" morph="robinson:ARAM">Eloi</w>, <w src="13" lemma="strong:G1682 lemma.TR:ελωι" morph="robinson:ARAM">Eloi</w>, <w src="14" lemma="strong:G2982 lemma.TR:λαμμα" morph="robinson:ARAM">lama</w> <w src="15" lemma="strong:G4518 lemma.TR:σαβαχθανι" morph="robinson:ARAM">sabachthani</w>?</q> <w src="16" lemma="strong:G3739 lemma.TR:ο" morph="robinson:R-NSN">which</w> <w src="17" lemma="strong:G1510 lemma.TR:εστιν" morph="robinson:V-PAI-3S">is</w>, <w src="18" lemma="strong:G3177 lemma.TR:μεθερμηνευομενον" morph="robinson:V-PPP-NSN">being interpreted</w>, <w src="21" lemma="strong:G1473 lemma.TR:μου" morph="robinson:P-1GS">My</w> <w src="19 20" lemma="strong:G3588 strong:G2316 lemma.TR:ο lemma.TR:θεος" morph="robinson:T-NSM robinson:N-NSM">God</w>, <w src="24" lemma="strong:G1473 lemma.TR:μου" morph="robinson:P-1GS">my</w> <w src="22 23" lemma="strong:G3588 strong:G2316 lemma.TR:ο lemma.TR:θεος" morph="robinson:T-NSM robinson:N-NSM">God</w>, <w src="25" lemma="strong:G1519 lemma.TR:εις" morph="robinson:PREP"/><w src="26" lemma="strong:G5101 lemma.TR:τι" morph="robinson:I-ASN">why</w> <w src="28" lemma="strong:G1459 lemma.TR:εγκατελιπες" morph="robinson:V-2AAI-2S">hast thou forsaken</w> <w src="27" lemma="strong:G1473 lemma.TR:με" morph="robinson:P-1AS">me</w>?<verse eID="Mark.15.34"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Mark.15.34" sID="Mark.15.34"/><w>And</w> <w>at the ninth</w> <w>hour</w> <w>Jesus</w> <w>cried</w> <w>with a loud</w> <w>voice</w>, <w>saying</w>, <q who="Jesus" marker=""><w>Eloi</w>, <w>Eloi</w>, <w>lama</w> <w>sabachthani</w>?</q> <w>which</w> <w>is</w>, <w>being interpreted</w>, <w>My</w> <w>God</w>, <w>my</w> <w>God</w>, <w/><w>why</w> <w>hast thou forsaken</w> <w>me</w>?<verse eID="Mark.15.34"/>
```

### LUK.4.17 (OSIS `Luke.4.17`)

- offset 18242426, length 1483, excerpt SHA-256 `91d73d04615d7ef879c18c008a49341d8d7c390e551d83332dd0a065f06ba426`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="Luke.4.17" sID="Luke.4.17"/><w src="1" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">And</w> <w src="2" lemma="strong:G1929 lemma.TR:επεδοθη" morph="robinson:V-API-3S">there was delivered</w> <w src="3" lemma="strong:G846 lemma.TR:αυτω" morph="robinson:P-DSM">unto him</w> <w src="4" lemma="strong:G975 lemma.TR:βιβλιον" morph="robinson:N-NSN">the book</w> <w src="6 7" lemma="strong:G3588 strong:G4396 lemma.TR:του lemma.TR:προφητου" morph="robinson:T-GSM robinson:N-GSM">of the prophet</w> <w src="5" lemma="strong:G2268 lemma.TR:ησαιου" morph="robinson:N-GSM">Esaias</w>. <w src="8" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">And</w> <w src="9" lemma="strong:G380 lemma.TR:αναπτυξας" morph="robinson:V-AAP-NSM">when he had opened</w> <w src="10 11" lemma="strong:G3588 strong:G975 lemma.TR:το lemma.TR:βιβλιον" morph="robinson:T-ASN robinson:N-ASN">the book</w>, <w src="12" lemma="strong:G2147 lemma.TR:ευρεν" morph="robinson:V-2AAI-3S">he found</w> <w src="13 14" lemma="strong:G3588 strong:G5117 lemma.TR:τον lemma.TR:τοπον" morph="robinson:T-ASM robinson:N-ASM">the place</w> <w src="15" lemma="strong:G3757 lemma.TR:ου" morph="robinson:ADV">where</w> <w src="16" lemma="strong:G1510 lemma.TR:ην" morph="robinson:V-IAI-3S">it was</w> <w src="17" lemma="strong:G1125 lemma.TR:γεγραμμενον" morph="robinson:V-RPP-NSN">written</w>,<verse eID="Luke.4.17"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Luke.4.17" sID="Luke.4.17"/><w>And</w> <w>there was delivered</w> <w>unto him</w> <w>the book</w> <w>of the prophet</w> <w>Esaias</w>. <w>And</w> <w>when he had opened</w> <w>the book</w>, <w>he found</w> <w>the place</w> <w>where</w> <w>it was</w> <w>written</w>,<verse eID="Luke.4.17"/>
```

### LUK.4.18 (OSIS `Luke.4.18`)

- offset 18243910, length 2592, excerpt SHA-256 `738b3abfae43513216556029a1418fb328392b17b65ad3f57f969df06520f1fd`
- `q` open at verse start: no
- `q` open at verse end: **yes** (continues into the next verse)
- `<q>` start tags in excerpt: 1; `</q>` end tags in excerpt: 0

```xml
<verse osisID="Luke.4.18" sID="Luke.4.18"/><q who="Jesus" marker=""><w src="1" lemma="strong:G4151 lemma.TR:πνευμα" morph="robinson:N-NSN">The Spirit</w> <w src="2" lemma="strong:G2962 lemma.TR:κυριου" morph="robinson:N-GSM">of the Lord</w> <transChange type="added">is</transChange> <w src="3" lemma="strong:G1909 lemma.TR:επ" morph="robinson:PREP">upon</w> <w src="4" lemma="strong:G1473 lemma.TR:εμε" morph="robinson:P-1AS">me</w>, <w src="6" lemma="strong:G1752 lemma.TR:ενεκεν" morph="robinson:PREP">because</w> <w src="7" lemma="strong:G5548 lemma.TR:εχρισεν" morph="robinson:V-AAI-3S">he hath anointed</w> <w src="8" lemma="strong:G1473 lemma.TR:με" morph="robinson:P-1AS">me</w> <w src="9" lemma="strong:G2097 lemma.TR:ευαγγελιζεσθαι" morph="robinson:V-PMN">to preach the gospel</w> <w src="10" lemma="strong:G4434 lemma.TR:πτωχοις" morph="robinson:A-DPM">to the poor</w>; <w src="11" lemma="strong:G649 lemma.TR:απεσταλκεν" morph="robinson:V-RAI-3S">he hath sent</w> <w src="12" lemma="strong:G1473 lemma.TR:με" morph="robinson:P-1AS">me</w> <w src="13" lemma="strong:G2390 lemma.TR:ιασασθαι" morph="robinson:V-ADN">to heal</w> <w src="15" lemma="strong:G4937 lemma.TR:συντετριμμενους" morph="robinson:V-RPP-APM">the brokenhearted</w>, <w src="16 17" lemma="strong:G3588 strong:G2588 lemma.TR:την lemma.TR:καρδιαν" morph="robinson:T-ASF robinson:N-ASF"/><w src="18" lemma="strong:G2784 lemma.TR:κηρυξαι" morph="robinson:V-AAN">to preach</w> <w src="20" lemma="strong:G859 lemma.TR:αφεσιν" morph="robinson:N-ASF">deliverance</w> <w src="19" lemma="strong:G164 lemma.TR:αιχμαλωτοις" morph="robinson:N-DPM">to the captives</w>, <w src="21" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="23" lemma="strong:G309 lemma.TR:αναβλεψιν" morph="robinson:N-ASF">recovering of sight</w> <w src="22" lemma="strong:G5185 lemma.TR:τυφλοις" morph="robinson:A-DPM">to the blind</w>, <w src="24" lemma="strong:G649 lemma.TR:αποστειλαι" morph="robinson:V-AAN">to set</w> <w src="26" lemma="strong:G1722 lemma.TR:εν" morph="robinson:PREP">at</w> <w src="27" lemma="strong:G859 lemma.TR:αφεσει" morph="robinson:N-DSF">liberty</w> <w src="25" lemma="strong:G2352 lemma.TR:τεθραυσμενους" morph="robinson:V-RPP-APM">them that are bruised</w>,<w src="5" lemma="strong:G3739 lemma.TR:ου" morph="robinson:R-GSN"/><w src="14" lemma="strong:G3588 lemma.TR:τους" morph="robinson:T-APM"/><verse eID="Luke.4.18"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Luke.4.18" sID="Luke.4.18"/><q who="Jesus" marker=""><w>The Spirit</w> <w>of the Lord</w> <transChange type="added">is</transChange> <w>upon</w> <w>me</w>, <w>because</w> <w>he hath anointed</w> <w>me</w> <w>to preach the gospel</w> <w>to the poor</w>; <w>he hath sent</w> <w>me</w> <w>to heal</w> <w>the brokenhearted</w>, <w/><w>to preach</w> <w>deliverance</w> <w>to the captives</w>, <w>and</w> <w>recovering of sight</w> <w>to the blind</w>, <w>to set</w> <w>at</w> <w>liberty</w> <w>them that are bruised</w>,<w/><w/><verse eID="Luke.4.18"/>
```

### LUK.4.19 (OSIS `Luke.4.19`)

- offset 18246503, length 443, excerpt SHA-256 `5eceaeccebe725798a90b1b6005d7d298f8e184295eb9d052d2fcff367e46ca5`
- `q` open at verse start: **yes**, its `<q>` tag is in `Luke.4.18` at byte 18243953
- `q` open at verse end: no
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 1

```xml
<verse osisID="Luke.4.19" sID="Luke.4.19"/><w src="1" lemma="strong:G2784 lemma.TR:κηρυξαι" morph="robinson:V-AAN">To preach</w> <w src="4" lemma="strong:G1184 lemma.TR:δεκτον" morph="robinson:A-ASM">the acceptable</w> <w src="2" lemma="strong:G1763 lemma.TR:ενιαυτον" morph="robinson:N-ASM">year</w> <w src="3" lemma="strong:G2962 lemma.TR:κυριου" morph="robinson:N-GSM">of the Lord</w>.</q><verse eID="Luke.4.19"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Luke.4.19" sID="Luke.4.19"/><w>To preach</w> <w>the acceptable</w> <w>year</w> <w>of the Lord</w>.</q><verse eID="Luke.4.19"/>
```

### LUK.4.20 (OSIS `Luke.4.20`)

- offset 18246947, length 1714, excerpt SHA-256 `a16affbc71fae1d3a255c7cbd34cb82a9e7d5bd64fce2c42904aeb9b1646951b`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="Luke.4.20" sID="Luke.4.20"/><w src="1" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">And</w> <w src="2" lemma="strong:G4428 lemma.TR:πτυξας" morph="robinson:V-AAP-NSM">he closed</w> <w src="3 4" lemma="strong:G3588 strong:G975 lemma.TR:το lemma.TR:βιβλιον" morph="robinson:T-ASN robinson:N-ASN">the book</w>, <w src="5" lemma="strong:G591 lemma.TR:αποδους" morph="robinson:V-2AAP-NSM" type="x-split-1402">and he gave</w> <transChange type="added">it</transChange> <w src="5" lemma="strong:G591 lemma.TR:αποδους" morph="robinson:V-2AAP-NSM" type="x-split-1402">again</w> <w src="6 7" lemma="strong:G3588 strong:G5257 lemma.TR:τω lemma.TR:υπηρετη" morph="robinson:T-DSM robinson:N-DSM">to the minister</w>, <w src="8" lemma="strong:G2523 lemma.TR:εκαθισεν" morph="robinson:V-AAI-3S">and sat down</w>. <w src="9" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">And</w> <w src="14 15" lemma="strong:G3588 strong:G3788 lemma.TR:οι lemma.TR:οφθαλμοι" morph="robinson:T-NPM robinson:N-NPM">the eyes</w> <w src="10" lemma="strong:G3956 lemma.TR:παντων" morph="robinson:A-GPM">of all them</w> <w src="16" lemma="strong:G1510 lemma.TR:ησαν" morph="robinson:V-IAI-3P">that were</w> <w src="11" lemma="strong:G1722 lemma.TR:εν" morph="robinson:PREP">in</w> <w src="12 13" lemma="strong:G3588 strong:G4864 lemma.TR:τη lemma.TR:συναγωγη" morph="robinson:T-DSF robinson:N-DSF">the synagogue</w> <w src="17" lemma="strong:G816 lemma.TR:ατενιζοντες" morph="robinson:V-PAP-NPM">were fastened</w> <w src="18" lemma="strong:G846 lemma.TR:αυτω" morph="robinson:P-DSM">on him</w>.<verse eID="Luke.4.20"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Luke.4.20" sID="Luke.4.20"/><w>And</w> <w>he closed</w> <w>the book</w>, <w>and he gave</w> <transChange type="added">it</transChange> <w>again</w> <w>to the minister</w>, <w>and sat down</w>. <w>And</w> <w>the eyes</w> <w>of all them</w> <w>that were</w> <w>in</w> <w>the synagogue</w> <w>were fastened</w> <w>on him</w>.<verse eID="Luke.4.20"/>
```

### LUK.4.21 (OSIS `Luke.4.21`)

- offset 18248662, length 1423, excerpt SHA-256 `cd666afad5227f4e54fc96095e434fe7dd67338633e1762b8357911d01b30c3a`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 1; `</q>` end tags in excerpt: 1

```xml
<verse osisID="Luke.4.21" sID="Luke.4.21"/><w src="6" lemma="strong:G3754 lemma.TR:οτι" morph="robinson:CONJ"/><w src="2" lemma="strong:G1161 lemma.TR:δε" morph="robinson:CONJ">And</w> <w src="1" lemma="strong:G756 lemma.TR:ηρξατο" morph="robinson:V-ADI-3S">he began</w> <w src="3" lemma="strong:G3004 lemma.TR:λεγειν" morph="robinson:V-PAN">to say</w> <w src="4" lemma="strong:G4314 lemma.TR:προς" morph="robinson:PREP">unto</w> <w src="5" lemma="strong:G846 lemma.TR:αυτους" morph="robinson:P-APM">them</w>, <q who="Jesus" marker=""><w src="7" lemma="strong:G4594 lemma.TR:σημερον" morph="robinson:ADV">This day</w> <w src="8" lemma="strong:G4137 lemma.TR:πεπληρωται" morph="robinson:V-RPI-3S" type="x-split-1403">is</w> <w src="11" lemma="strong:G3778 lemma.TR:αυτη" morph="robinson:D-NSF">this</w> <w src="9 10" lemma="strong:G3588 strong:G1124 lemma.TR:η lemma.TR:γραφη" morph="robinson:T-NSF robinson:N-NSF">scripture</w> <w src="8" lemma="strong:G4137 lemma.TR:πεπληρωται" morph="robinson:V-RPI-3S" type="x-split-1403">fulfilled</w> <w src="12" lemma="strong:G1722 lemma.TR:εν" morph="robinson:PREP">in</w> <w src="15" lemma="strong:G4771 lemma.TR:υμων" morph="robinson:P-2GP">your</w> <w src="13 14" lemma="strong:G3588 strong:G3775 lemma.TR:τοις lemma.TR:ωσιν" morph="robinson:T-DPN robinson:N-DPN">ears</w>.</q><verse eID="Luke.4.21"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Luke.4.21" sID="Luke.4.21"/><w/><w>And</w> <w>he began</w> <w>to say</w> <w>unto</w> <w>them</w>, <q who="Jesus" marker=""><w>This day</w> <w>is</w> <w>this</w> <w>scripture</w> <w>fulfilled</w> <w>in</w> <w>your</w> <w>ears</w>.</q><verse eID="Luke.4.21"/>
```

### LUK.23.46 (OSIS `Luke.23.46`)

- offset 19709410, length 1741, excerpt SHA-256 `5811cadf3e0b28ef36d44a504e7d530cf59fa23606639d6b2d77e69e889a345e`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 1; `</q>` end tags in excerpt: 1

```xml
<verse osisID="Luke.23.46" sID="Luke.23.46"/><milestone type="x-p" marker="¶"/><w src="1" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">And</w> <w src="5 6" lemma="strong:G3588 strong:G2424 lemma.TR:ο lemma.TR:ιησους" morph="robinson:T-NSM robinson:N-NSM">when Jesus</w> <w src="2" lemma="strong:G5455 lemma.TR:φωνησας" morph="robinson:V-AAP-NSM">had cried</w> <w src="4" lemma="strong:G3173 lemma.TR:μεγαλη" morph="robinson:A-DSF">with a loud</w> <w src="3" lemma="strong:G5456 lemma.TR:φωνη" morph="robinson:N-DSF">voice</w>, <w src="7" lemma="strong:G3004 lemma.TR:ειπεν" morph="robinson:V-2AAI-3S">he said</w>, <q who="Jesus" marker=""><w src="8" lemma="strong:G3962 lemma.TR:πατερ" morph="robinson:N-VSM">Father</w>, <w src="9" lemma="strong:G1519 lemma.TR:εις" morph="robinson:PREP">into</w> <w src="11" lemma="strong:G4771 lemma.TR:σου" morph="robinson:P-2GS">thy</w> <w src="10" lemma="strong:G5495 lemma.TR:χειρας" morph="robinson:N-APF">hands</w> <w src="12" lemma="strong:G3908 lemma.TR:παραθησομαι" morph="robinson:V-FDI-1S">I commend</w> <w src="15" lemma="strong:G1473 lemma.TR:μου" morph="robinson:P-1GS">my</w> <w src="13 14" lemma="strong:G3588 strong:G4151 lemma.TR:το lemma.TR:πνευμα" morph="robinson:T-ASN robinson:N-ASN">spirit</w>:</q> <w src="16" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="18" lemma="strong:G3004 lemma.TR:ειπων" morph="robinson:V-2AAP-NSM">having said</w> <w src="17" lemma="strong:G3778 lemma.TR:ταυτα" morph="robinson:D-APN">thus</w>, <w src="19" lemma="strong:G1606 lemma.TR:εξεπνευσεν" morph="robinson:V-AAI-3S">he gave up the ghost</w>.<verse eID="Luke.23.46"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Luke.23.46" sID="Luke.23.46"/><milestone type="x-p" marker="¶"/><w>And</w> <w>when Jesus</w> <w>had cried</w> <w>with a loud</w> <w>voice</w>, <w>he said</w>, <q who="Jesus" marker=""><w>Father</w>, <w>into</w> <w>thy</w> <w>hands</w> <w>I commend</w> <w>my</w> <w>spirit</w>:</q> <w>and</w> <w>having said</w> <w>thus</w>, <w>he gave up the ghost</w>.<verse eID="Luke.23.46"/>
```

### LUK.23.47 (OSIS `Luke.23.47`)

- offset 19711152, length 1369, excerpt SHA-256 `b20696360912901181ea5c620d60410c69b8fb52e15c671ee1373c159c071660`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="Luke.23.47" sID="Luke.23.47"/><w src="2" lemma="strong:G1161 lemma.TR:δε" morph="robinson:CONJ">Now</w> <w src="3 4" lemma="strong:G3588 strong:G1543 lemma.TR:ο lemma.TR:εκατονταρχος" morph="robinson:T-NSM robinson:N-NSM">when the centurion</w> <w src="1" lemma="strong:G3708 lemma.TR:ιδων" morph="robinson:V-2AAP-NSM">saw</w> <w src="5" lemma="strong:G3588 lemma.TR:το" morph="robinson:T-ASN">what</w> <w src="6" lemma="strong:G1096 lemma.TR:γενομενον" morph="robinson:V-2ADP-ASN">was done</w>, <w src="7" lemma="strong:G1392 lemma.TR:εδοξασεν" morph="robinson:V-AAI-3S">he glorified</w> <w src="8 9" lemma="strong:G3588 strong:G2316 lemma.TR:τον lemma.TR:θεον" morph="robinson:T-ASM robinson:N-ASM">God</w>, <w src="10" lemma="strong:G3004 lemma.TR:λεγων" morph="robinson:V-PAP-NSM">saying</w>, <w src="11" lemma="strong:G3689 lemma.TR:οντως" morph="robinson:ADV">Certainly</w> <w src="14" lemma="strong:G3778 lemma.TR:ουτος" morph="robinson:D-NSM">this</w> <w src="16" lemma="strong:G1510 lemma.TR:ην" morph="robinson:V-IAI-3S">was</w> <w src="15" lemma="strong:G1342 lemma.TR:δικαιος" morph="robinson:A-NSM">a righteous</w> <w src="12 13" lemma="strong:G3588 strong:G444 lemma.TR:ο lemma.TR:ανθρωπος" morph="robinson:T-NSM robinson:N-NSM">man</w>.<verse eID="Luke.23.47"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Luke.23.47" sID="Luke.23.47"/><w>Now</w> <w>when the centurion</w> <w>saw</w> <w>what</w> <w>was done</w>, <w>he glorified</w> <w>God</w>, <w>saying</w>, <w>Certainly</w> <w>this</w> <w>was</w> <w>a righteous</w> <w>man</w>.<verse eID="Luke.23.47"/>
```

### JHN.3.2 (OSIS `John.3.2`)

- offset 19915651, length 2886, excerpt SHA-256 `3f16084d123faf3caacfebabd176b4cef8c864112a0dc713e1a0a2bd992da73c`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="John.3.2" sID="John.3.2"/><w src="1" lemma="strong:G3778 lemma.TR:ουτος" morph="robinson:D-NSM">The same</w> <w src="2" lemma="strong:G2064 lemma.TR:ηλθεν" morph="robinson:V-2AAI-3S">came</w> <w src="3" lemma="strong:G4314 lemma.TR:προς" morph="robinson:PREP">to</w> <w src="4 5" lemma="strong:G3588 strong:G2424 lemma.TR:τον lemma.TR:ιησουν" morph="robinson:T-ASM robinson:N-ASM">Jesus</w> <w src="6" lemma="strong:G3571 lemma.TR:νυκτος" morph="robinson:N-GSF">by night</w>, <w src="7" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="8" lemma="strong:G3004 lemma.TR:ειπεν" morph="robinson:V-2AAI-3S">said</w> <w src="9" lemma="strong:G846 lemma.TR:αυτω" morph="robinson:P-DSM">unto him</w>, <w src="10" lemma="strong:G4461 lemma.TR:ραββι" morph="robinson:HEB">Rabbi</w>, <w src="11" lemma="strong:G1492 lemma.TR:οιδαμεν" morph="robinson:V-RAI-1P">we know</w> <w src="12" lemma="strong:G3754 lemma.TR:οτι" morph="robinson:CONJ">that</w> <w src="15" lemma="strong:G2064 lemma.TR:εληλυθας" morph="robinson:V-2RAI-2S" type="x-split-1790">thou art</w> <w src="16" lemma="strong:G1320 lemma.TR:διδασκαλος" morph="robinson:N-NSM">a teacher</w> <w src="15" lemma="strong:G2064 lemma.TR:εληλυθας" morph="robinson:V-2RAI-2S" type="x-split-1790">come</w> <w src="13" lemma="strong:G575 lemma.TR:απο" morph="robinson:PREP">from</w> <w src="14" lemma="strong:G2316 lemma.TR:θεου" morph="robinson:N-GSM">God</w>: <w src="18" lemma="strong:G1063 lemma.TR:γαρ" morph="robinson:CONJ">for</w> <w src="17" lemma="strong:G3762 lemma.TR:ουδεις" morph="robinson:A-NSM-N">no man</w> <w src="22" lemma="strong:G1410 lemma.TR:δυναται" morph="robinson:V-PNI-3S">can</w> <w src="23" lemma="strong:G4160 lemma.TR:ποιειν" morph="robinson:V-PAN">do</w> <w src="19" lemma="strong:G3778 lemma.TR:ταυτα" morph="robinson:D-APN">these</w> <w src="20 21" lemma="strong:G3588 strong:G4592 lemma.TR:τα lemma.TR:σημεια" morph="robinson:T-APN robinson:N-APN">miracles</w> <w src="24" lemma="strong:G3739 lemma.TR:α" morph="robinson:R-APN">that</w> <w src="25" lemma="strong:G4771 lemma.TR:συ" morph="robinson:P-2NS">thou</w> <w src="26" lemma="strong:G4160 lemma.TR:ποιεις" morph="robinson:V-PAI-2S">doest</w>, <w src="27" lemma="strong:G1437 lemma.TR:εαν" morph="robinson:COND"/><w src="28" lemma="strong:G3361 lemma.TR:μη" morph="robinson:PRT-N">except</w> <w src="30 31" lemma="strong:G3588 strong:G2316 lemma.TR:ο lemma.TR:θεος" morph="robinson:T-NSM robinson:N-NSM">God</w> <w src="29" lemma="strong:G1510 lemma.TR:η" morph="robinson:V-PAS-3S">be</w> <w src="32" lemma="strong:G3326 lemma.TR:μετ" morph="robinson:PREP">with</w> <w src="33" lemma="strong:G846 lemma.TR:αυτου" morph="robinson:P-GSM">him</w>.<verse eID="John.3.2"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="John.3.2" sID="John.3.2"/><w>The same</w> <w>came</w> <w>to</w> <w>Jesus</w> <w>by night</w>, <w>and</w> <w>said</w> <w>unto him</w>, <w>Rabbi</w>, <w>we know</w> <w>that</w> <w>thou art</w> <w>a teacher</w> <w>come</w> <w>from</w> <w>God</w>: <w>for</w> <w>no man</w> <w>can</w> <w>do</w> <w>these</w> <w>miracles</w> <w>that</w> <w>thou</w> <w>doest</w>, <w/><w>except</w> <w>God</w> <w>be</w> <w>with</w> <w>him</w>.<verse eID="John.3.2"/>
```

### JHN.3.3 (OSIS `John.3.3`)

- offset 19918538, length 1828, excerpt SHA-256 `cc6bdd66c33d5ec62454e1158c9fc17c5efd6996e2037a799c5d2f4ece6f83de`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 1; `</q>` end tags in excerpt: 1

```xml
<verse osisID="John.3.3" sID="John.3.3"/><w src="2 3" lemma="strong:G3588 strong:G2424 lemma.TR:ο lemma.TR:ιησους" morph="robinson:T-NSM robinson:N-NSM">Jesus</w> <w src="1" lemma="strong:G611 lemma.TR:απεκριθη" morph="robinson:V-ADI-3S">answered</w> <w src="4" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="5" lemma="strong:G3004 lemma.TR:ειπεν" morph="robinson:V-2AAI-3S">said</w> <w src="6" lemma="strong:G846 lemma.TR:αυτω" morph="robinson:P-DSM">unto him</w>, <q who="Jesus" marker=""><w src="7" lemma="strong:G281 lemma.TR:αμην" morph="robinson:HEB">Verily</w>, <w src="8" lemma="strong:G281 lemma.TR:αμην" morph="robinson:HEB">verily</w>, <w src="9" lemma="strong:G3004 lemma.TR:λεγω" morph="robinson:V-PAI-1S">I say</w> <w src="10" lemma="strong:G4771 lemma.TR:σοι" morph="robinson:P-2DS">unto thee</w>, <w src="11" lemma="strong:G1437 lemma.TR:εαν" morph="robinson:COND"/><w src="12" lemma="strong:G3361 lemma.TR:μη" morph="robinson:PRT-N">Except</w> <w src="13" lemma="strong:G5100 lemma.TR:τις" morph="robinson:X-NSM">a man</w> <w src="14" lemma="strong:G1080 lemma.TR:γεννηθη" morph="robinson:V-APS-3S">be born</w> <w src="15" lemma="strong:G509 lemma.TR:ανωθεν" morph="robinson:ADV">again</w>, <w src="16 17" lemma="strong:G3756 strong:G1410 lemma.TR:ου lemma.TR:δυναται" morph="robinson:PRT-N robinson:V-PNI-3S">he cannot</w> <w src="18" lemma="strong:G3708 lemma.TR:ιδειν" morph="robinson:V-2AAN">see</w> <w src="19 20" lemma="strong:G3588 strong:G932 lemma.TR:την lemma.TR:βασιλειαν" morph="robinson:T-ASF robinson:N-ASF">the kingdom</w> <w src="21 22" lemma="strong:G3588 strong:G2316 lemma.TR:του lemma.TR:θεου" morph="robinson:T-GSM robinson:N-GSM">of God</w>.</q><verse eID="John.3.3"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="John.3.3" sID="John.3.3"/><w>Jesus</w> <w>answered</w> <w>and</w> <w>said</w> <w>unto him</w>, <q who="Jesus" marker=""><w>Verily</w>, <w>verily</w>, <w>I say</w> <w>unto thee</w>, <w/><w>Except</w> <w>a man</w> <w>be born</w> <w>again</w>, <w>he cannot</w> <w>see</w> <w>the kingdom</w> <w>of God</w>.</q><verse eID="John.3.3"/>
```

### JHN.3.4 (OSIS `John.3.4`)

- offset 19920367, length 1962, excerpt SHA-256 `2d4345d26eb46ca770d7f2b7789c96144528e73d1c29acfeaa334864ce0f96b5`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="John.3.4" sID="John.3.4"/><w src="4 5" lemma="strong:G3588 strong:G3530 lemma.TR:ο lemma.TR:νικοδημος" morph="robinson:T-NSM robinson:N-NSM">Nicodemus</w> <w src="1" lemma="strong:G3004 lemma.TR:λεγει" morph="robinson:V-PAI-3S">saith</w> <w src="2" lemma="strong:G4314 lemma.TR:προς" morph="robinson:PREP">unto</w> <w src="3" lemma="strong:G846 lemma.TR:αυτον" morph="robinson:P-ASM">him</w>, <w src="6" lemma="strong:G4459 lemma.TR:πως" morph="robinson:ADV-I">How</w> <w src="7" lemma="strong:G1410 lemma.TR:δυναται" morph="robinson:V-PNI-3S">can</w> <w src="8" lemma="strong:G444 lemma.TR:ανθρωπος" morph="robinson:N-NSM">a man</w> <w src="9" lemma="strong:G1080 lemma.TR:γεννηθηναι" morph="robinson:V-APN">be born</w> <w src="11" lemma="strong:G1510 lemma.TR:ων" morph="robinson:V-PAP-NSM">when he is</w> <w src="10" lemma="strong:G1088 lemma.TR:γερων" morph="robinson:N-NSM">old</w>? <w src="12" lemma="strong:G3361 lemma.TR:μη" morph="robinson:PRT-N"/><w src="13" lemma="strong:G1410 lemma.TR:δυναται" morph="robinson:V-PNI-3S">can he</w> <w src="21" lemma="strong:G1525 lemma.TR:εισελθειν" morph="robinson:V-2AAN">enter</w> <w src="20" lemma="strong:G1208 lemma.TR:δευτερον" morph="robinson:ADV">the second time</w> <w src="14" lemma="strong:G1519 lemma.TR:εις" morph="robinson:PREP">into</w> <w src="19" lemma="strong:G846 lemma.TR:αυτου" morph="robinson:P-GSM">his</w> <w src="17 18" lemma="strong:G3588 strong:G3384 lemma.TR:της lemma.TR:μητρος" morph="robinson:T-GSF robinson:N-GSF">mother’s</w> <w src="15 16" lemma="strong:G3588 strong:G2836 lemma.TR:την lemma.TR:κοιλιαν" morph="robinson:T-ASF robinson:N-ASF">womb</w>, <w src="22" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="23" lemma="strong:G1080 lemma.TR:γεννηθηναι" morph="robinson:V-APN">be born</w>?<verse eID="John.3.4"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="John.3.4" sID="John.3.4"/><w>Nicodemus</w> <w>saith</w> <w>unto</w> <w>him</w>, <w>How</w> <w>can</w> <w>a man</w> <w>be born</w> <w>when he is</w> <w>old</w>? <w/><w>can he</w> <w>enter</w> <w>the second time</w> <w>into</w> <w>his</w> <w>mother’s</w> <w>womb</w>, <w>and</w> <w>be born</w>?<verse eID="John.3.4"/>
```

### JHN.3.5 (OSIS `John.3.5`)

- offset 19922330, length 1934, excerpt SHA-256 `a16dc7b2a788ae8258334d3ed82e16436da3cb54f13d4d27feb1baf793aa9a06`
- `q` open at verse start: no
- `q` open at verse end: **yes** (continues into the next verse)
- `<q>` start tags in excerpt: 1; `</q>` end tags in excerpt: 0

```xml
<verse osisID="John.3.5" sID="John.3.5"/><w src="2 3" lemma="strong:G3588 strong:G2424 lemma.TR:ο lemma.TR:ιησους" morph="robinson:T-NSM robinson:N-NSM">Jesus</w> <w src="1" lemma="strong:G611 lemma.TR:απεκριθη" morph="robinson:V-ADI-3S">answered</w>, <q who="Jesus" marker=""><w src="4" lemma="strong:G281 lemma.TR:αμην" morph="robinson:HEB">Verily</w>, <w src="5" lemma="strong:G281 lemma.TR:αμην" morph="robinson:HEB">verily</w>, <w src="6" lemma="strong:G3004 lemma.TR:λεγω" morph="robinson:V-PAI-1S">I say</w> <w src="7" lemma="strong:G4771 lemma.TR:σοι" morph="robinson:P-2DS">unto thee</w>, <w src="8 9" lemma="strong:G1437 strong:G3361 lemma.TR:εαν lemma.TR:μη" morph="robinson:COND robinson:PRT-N">Except</w> <w src="10" lemma="strong:G5100 lemma.TR:τις" morph="robinson:X-NSM">a man</w> <w src="11" lemma="strong:G1080 lemma.TR:γεννηθη" morph="robinson:V-APS-3S">be born</w> <w src="12" lemma="strong:G1537 lemma.TR:εξ" morph="robinson:PREP">of</w> <w src="13" lemma="strong:G5204 lemma.TR:υδατος" morph="robinson:N-GSN">water</w> <w src="14" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <transChange type="added">of</transChange> <w src="15" lemma="strong:G4151 lemma.TR:πνευματος" morph="robinson:N-GSN">the Spirit</w>, <w src="16 17" lemma="strong:G3756 strong:G1410 lemma.TR:ου lemma.TR:δυναται" morph="robinson:PRT-N robinson:V-PNI-3S">he cannot</w> <w src="18" lemma="strong:G1525 lemma.TR:εισελθειν" morph="robinson:V-2AAN">enter</w> <w src="19" lemma="strong:G1519 lemma.TR:εις" morph="robinson:PREP">into</w> <w src="20 21" lemma="strong:G3588 strong:G932 lemma.TR:την lemma.TR:βασιλειαν" morph="robinson:T-ASF robinson:N-ASF">the kingdom</w> <w src="22 23" lemma="strong:G3588 strong:G2316 lemma.TR:του lemma.TR:θεου" morph="robinson:T-GSM robinson:N-GSM">of God</w>.<verse eID="John.3.5"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="John.3.5" sID="John.3.5"/><w>Jesus</w> <w>answered</w>, <q who="Jesus" marker=""><w>Verily</w>, <w>verily</w>, <w>I say</w> <w>unto thee</w>, <w>Except</w> <w>a man</w> <w>be born</w> <w>of</w> <w>water</w> <w>and</w> <transChange type="added">of</transChange> <w>the Spirit</w>, <w>he cannot</w> <w>enter</w> <w>into</w> <w>the kingdom</w> <w>of God</w>.<verse eID="John.3.5"/>
```

### JHN.3.6 (OSIS `John.3.6`)

- offset 19924265, length 1293, excerpt SHA-256 `8bd138084077e73915ad6f7f65ef250cf8c100d7e647130c5f227b4bd57053a1`
- `q` open at verse start: **yes**, its `<q>` tag is in `John.3.5` at byte 19922597
- `q` open at verse end: **yes** (continues into the next verse)
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="John.3.6" sID="John.3.6"/><w src="9" lemma="strong:G3588 lemma.TR:το" morph="robinson:T-NSN"/><w src="1" lemma="strong:G3588 lemma.TR:το" morph="robinson:T-NSN"/><w src="2" lemma="strong:G1080 lemma.TR:γεγεννημενον" morph="robinson:V-RPP-NSN">That which is born</w> <w src="3" lemma="strong:G1537 lemma.TR:εκ" morph="robinson:PREP">of</w> <w src="4 5" lemma="strong:G3588 strong:G4561 lemma.TR:της lemma.TR:σαρκος" morph="robinson:T-GSF robinson:N-GSF">the flesh</w> <w src="7" lemma="strong:G1510 lemma.TR:εστιν" morph="robinson:V-PAI-3S">is</w> <w src="6" lemma="strong:G4561 lemma.TR:σαρξ" morph="robinson:N-NSF">flesh</w>; <w src="8" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="10" lemma="strong:G1080 lemma.TR:γεγεννημενον" morph="robinson:V-RPP-NSN">that which is born</w> <w src="11" lemma="strong:G1537 lemma.TR:εκ" morph="robinson:PREP">of</w> <w src="12 13" lemma="strong:G3588 strong:G4151 lemma.TR:του lemma.TR:πνευματος" morph="robinson:T-GSN robinson:N-GSN">the Spirit</w> <w src="15" lemma="strong:G1510 lemma.TR:εστιν" morph="robinson:V-PAI-3S">is</w> <w src="14" lemma="strong:G4151 lemma.TR:πνευμα" morph="robinson:N-NSN">spirit</w>.<verse eID="John.3.6"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="John.3.6" sID="John.3.6"/><w/><w/><w>That which is born</w> <w>of</w> <w>the flesh</w> <w>is</w> <w>flesh</w>; <w>and</w> <w>that which is born</w> <w>of</w> <w>the Spirit</w> <w>is</w> <w>spirit</w>.<verse eID="John.3.6"/>
```

### JHN.3.7 (OSIS `John.3.7`)

- offset 19925559, length 835, excerpt SHA-256 `66f48a0f73552bb105f1593494df6c045954ff06e9284c01e14190cfa0236e8e`
- `q` open at verse start: **yes**, its `<q>` tag is in `John.3.5` at byte 19922597
- `q` open at verse end: **yes** (continues into the next verse)
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="John.3.7" sID="John.3.7"/><w src="2" lemma="strong:G2296 lemma.TR:θαυμασης" morph="robinson:V-AAS-2S">Marvel</w> <w src="1" lemma="strong:G3361 lemma.TR:μη" morph="robinson:PRT-N">not</w> <w src="3" lemma="strong:G3754 lemma.TR:οτι" morph="robinson:CONJ">that</w> <w src="4" lemma="strong:G3004 lemma.TR:ειπον" morph="robinson:V-2AAI-1S">I said</w> <w src="5" lemma="strong:G4771 lemma.TR:σοι" morph="robinson:P-2DS">unto thee</w>, <w src="7" lemma="strong:G4771 lemma.TR:υμας" morph="robinson:P-2AP">Ye</w> <w src="6" lemma="strong:G1163 lemma.TR:δει" morph="robinson:V-PAI-3S">must</w> <w src="8" lemma="strong:G1080 lemma.TR:γεννηθηναι" morph="robinson:V-APN">be born</w> <w src="9" lemma="strong:G509 lemma.TR:ανωθεν" morph="robinson:ADV">again</w>.<verse eID="John.3.7"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="John.3.7" sID="John.3.7"/><w>Marvel</w> <w>not</w> <w>that</w> <w>I said</w> <w>unto thee</w>, <w>Ye</w> <w>must</w> <w>be born</w> <w>again</w>.<verse eID="John.3.7"/>
```

### JHN.3.8 (OSIS `John.3.8`)

- offset 19926395, length 2337, excerpt SHA-256 `aa58e0f14857b43f1f7b84985cf19103b2619bbb2b7ca0711f4cf5100cb321f9`
- `q` open at verse start: **yes**, its `<q>` tag is in `John.3.5` at byte 19922597
- `q` open at verse end: no
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 1

```xml
<verse osisID="John.3.8" sID="John.3.8"/><w src="22" lemma="strong:G3588 lemma.TR:ο" morph="robinson:T-NSM"/><w src="1 2" lemma="strong:G3588 strong:G4151 lemma.TR:το lemma.TR:πνευμα" morph="robinson:T-NSN robinson:N-NSN">The wind</w> <w src="5" lemma="strong:G4154 lemma.TR:πνει" morph="robinson:V-PAI-3S">bloweth</w> <w src="3" lemma="strong:G3699 lemma.TR:οπου" morph="robinson:ADV">where</w> <w src="4" lemma="strong:G2309 lemma.TR:θελει" morph="robinson:V-PAI-3S">it listeth</w>, <w src="6" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="10" lemma="strong:G191 lemma.TR:ακουεις" morph="robinson:V-PAI-2S">thou hearest</w> <w src="7 8" lemma="strong:G3588 strong:G5456 lemma.TR:την lemma.TR:φωνην" morph="robinson:T-ASF robinson:N-ASF">the sound</w> <w src="9" lemma="strong:G846 lemma.TR:αυτου" morph="robinson:P-GSN">thereof</w>, <w src="11" lemma="strong:G235 lemma.TR:αλλ" morph="robinson:CONJ">but</w> <w src="13" lemma="strong:G1492 lemma.TR:οιδας" morph="robinson:V-RAI-2S" type="x-split-1791">canst</w> <w src="12" lemma="strong:G3756 lemma.TR:ουκ" morph="robinson:PRT-N">not</w> <w src="13" lemma="strong:G1492 lemma.TR:οιδας" morph="robinson:V-RAI-2S" type="x-split-1791">tell</w> <w src="14" lemma="strong:G4159 lemma.TR:ποθεν" morph="robinson:ADV-I">whence</w> <w src="15" lemma="strong:G2064 lemma.TR:ερχεται" morph="robinson:V-PNI-3S">it cometh</w>, <w src="16" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="17" lemma="strong:G4226 lemma.TR:που" morph="robinson:ADV-I">whither</w> <w src="18" lemma="strong:G5217 lemma.TR:υπαγει" morph="robinson:V-PAI-3S">it goeth</w>: <w src="19" lemma="strong:G3779 lemma.TR:ουτως" morph="robinson:ADV">so</w> <w src="20" lemma="strong:G1510 lemma.TR:εστιν" morph="robinson:V-PAI-3S">is</w> <w src="21" lemma="strong:G3956 lemma.TR:πας" morph="robinson:A-NSM">every</w> <w src="23" lemma="strong:G1080 lemma.TR:γεγεννημενος" morph="robinson:V-RPP-NSM">one that is born</w> <w src="24" lemma="strong:G1537 lemma.TR:εκ" morph="robinson:PREP">of</w> <w src="25 26" lemma="strong:G3588 strong:G4151 lemma.TR:του lemma.TR:πνευματος" morph="robinson:T-GSN robinson:N-GSN">the Spirit</w>.</q><verse eID="John.3.8"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="John.3.8" sID="John.3.8"/><w/><w>The wind</w> <w>bloweth</w> <w>where</w> <w>it listeth</w>, <w>and</w> <w>thou hearest</w> <w>the sound</w> <w>thereof</w>, <w>but</w> <w>canst</w> <w>not</w> <w>tell</w> <w>whence</w> <w>it cometh</w>, <w>and</w> <w>whither</w> <w>it goeth</w>: <w>so</w> <w>is</w> <w>every</w> <w>one that is born</w> <w>of</w> <w>the Spirit</w>.</q><verse eID="John.3.8"/>
```

### JHN.3.9 (OSIS `John.3.9`)

- offset 19928733, length 859, excerpt SHA-256 `60ddb8acd1cacc20e37c67de1aeb6b20468863bc5276fe3d007a022720352bdf`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="John.3.9" sID="John.3.9"/><w src="2" lemma="strong:G3530 lemma.TR:νικοδημος" morph="robinson:N-NSM">Nicodemus</w> <w src="1" lemma="strong:G611 lemma.TR:απεκριθη" morph="robinson:V-ADI-3S">answered</w> <w src="3" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="4" lemma="strong:G3004 lemma.TR:ειπεν" morph="robinson:V-2AAI-3S">said</w> <w src="5" lemma="strong:G846 lemma.TR:αυτω" morph="robinson:P-DSM">unto him</w>, <w src="6" lemma="strong:G4459 lemma.TR:πως" morph="robinson:ADV-I">How</w> <w src="7" lemma="strong:G1410 lemma.TR:δυναται" morph="robinson:V-PNI-3S">can</w> <w src="8" lemma="strong:G3778 lemma.TR:ταυτα" morph="robinson:D-NPN">these things</w> <w src="9" lemma="strong:G1096 lemma.TR:γενεσθαι" morph="robinson:V-2ADN">be</w>?<verse eID="John.3.9"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="John.3.9" sID="John.3.9"/><w>Nicodemus</w> <w>answered</w> <w>and</w> <w>said</w> <w>unto him</w>, <w>How</w> <w>can</w> <w>these things</w> <w>be</w>?<verse eID="John.3.9"/>
```

### JHN.3.10 (OSIS `John.3.10`)

- offset 19929593, length 1383, excerpt SHA-256 `20bad009b27dc4470bad693850d900860ddca6624155c1323e6bfe56e5fbbbd8`
- `q` open at verse start: no
- `q` open at verse end: **yes** (continues into the next verse)
- `<q>` start tags in excerpt: 1; `</q>` end tags in excerpt: 0

```xml
<verse osisID="John.3.10" sID="John.3.10"/><w src="11" lemma="strong:G3588 lemma.TR:του" morph="robinson:T-GSM"/><w src="2 3" lemma="strong:G3588 strong:G2424 lemma.TR:ο lemma.TR:ιησους" morph="robinson:T-NSM robinson:N-NSM">Jesus</w> <w src="1" lemma="strong:G611 lemma.TR:απεκριθη" morph="robinson:V-ADI-3S">answered</w> <w src="4" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="5" lemma="strong:G3004 lemma.TR:ειπεν" morph="robinson:V-2AAI-3S">said</w> <w src="6" lemma="strong:G846 lemma.TR:αυτω" morph="robinson:P-DSM">unto him</w>, <q who="Jesus" marker=""><w src="8" lemma="strong:G1510 lemma.TR:ει" morph="robinson:V-PAI-2S">Art</w> <w src="7" lemma="strong:G4771 lemma.TR:συ" morph="robinson:P-2NS">thou</w> <w src="9 10" lemma="strong:G3588 strong:G1320 lemma.TR:ο lemma.TR:διδασκαλος" morph="robinson:T-NSM robinson:N-NSM">a master</w> <w src="12" lemma="strong:G2474 lemma.TR:ισραηλ" morph="robinson:N-PRI">of Israel</w>, <w src="13" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="16" lemma="strong:G1097 lemma.TR:γινωσκεις" morph="robinson:V-PAI-2S">knowest</w> <w src="15" lemma="strong:G3756 lemma.TR:ου" morph="robinson:PRT-N">not</w> <w src="14" lemma="strong:G3778 lemma.TR:ταυτα" morph="robinson:D-APN">these things</w>?<verse eID="John.3.10"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="John.3.10" sID="John.3.10"/><w/><w>Jesus</w> <w>answered</w> <w>and</w> <w>said</w> <w>unto him</w>, <q who="Jesus" marker=""><w>Art</w> <w>thou</w> <w>a master</w> <w>of Israel</w>, <w>and</w> <w>knowest</w> <w>not</w> <w>these things</w>?<verse eID="John.3.10"/>
```

### JHN.3.11 (OSIS `John.3.11`)

- offset 19930977, length 1596, excerpt SHA-256 `07acb77412b22fa09a715ecfea5473a95366300fc6ec2cbd04e31dc31dcba68e`
- `q` open at verse start: **yes**, its `<q>` tag is in `John.3.10` at byte 19930186
- `q` open at verse end: **yes** (continues into the next verse)
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="John.3.11" sID="John.3.11"/><w src="1" lemma="strong:G281 lemma.TR:αμην" morph="robinson:HEB">Verily</w>, <w src="2" lemma="strong:G281 lemma.TR:αμην" morph="robinson:HEB">verily</w>, <w src="3" lemma="strong:G3004 lemma.TR:λεγω" morph="robinson:V-PAI-1S">I say</w> <w src="4" lemma="strong:G4771 lemma.TR:σοι" morph="robinson:P-2DS">unto thee</w>, <w src="5" lemma="strong:G3754 lemma.TR:οτι" morph="robinson:CONJ"/><w src="8" lemma="strong:G2980 lemma.TR:λαλουμεν" morph="robinson:V-PAI-1P">We speak</w> <w src="6" lemma="strong:G3739 lemma.TR:ο" morph="robinson:R-ASN">that</w> <w src="7" lemma="strong:G1492 lemma.TR:οιδαμεν" morph="robinson:V-RAI-1P">we do know</w>, <w src="9" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="12" lemma="strong:G3140 lemma.TR:μαρτυρουμεν" morph="robinson:V-PAI-1P">testify</w> <w src="10" lemma="strong:G3739 lemma.TR:ο" morph="robinson:R-ASN">that</w> <w src="11" lemma="strong:G3708 lemma.TR:εωρακαμεν" morph="robinson:V-RAI-1P-ATT">we have seen</w>; <w src="13" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="18" lemma="strong:G2983 lemma.TR:λαμβανετε" morph="robinson:V-PAI-2P">ye receive</w> <w src="17" lemma="strong:G3756 lemma.TR:ου" morph="robinson:PRT-N">not</w> <w src="16" lemma="strong:G1473 lemma.TR:ημων" morph="robinson:P-1GP">our</w> <w src="14 15" lemma="strong:G3588 strong:G3141 lemma.TR:την lemma.TR:μαρτυριαν" morph="robinson:T-ASF robinson:N-ASF">witness</w>.<verse eID="John.3.11"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="John.3.11" sID="John.3.11"/><w>Verily</w>, <w>verily</w>, <w>I say</w> <w>unto thee</w>, <w/><w>We speak</w> <w>that</w> <w>we do know</w>, <w>and</w> <w>testify</w> <w>that</w> <w>we have seen</w>; <w>and</w> <w>ye receive</w> <w>not</w> <w>our</w> <w>witness</w>.<verse eID="John.3.11"/>
```

### JHN.3.12 (OSIS `John.3.12`)

- offset 19932574, length 1401, excerpt SHA-256 `d46c29e95533f66267ca586191457bd67a8235e58d592204d9b9689555a04419`
- `q` open at verse start: **yes**, its `<q>` tag is in `John.3.10` at byte 19930186
- `q` open at verse end: **yes** (continues into the next verse)
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="John.3.12" sID="John.3.12"/><w src="13" lemma="strong:G3588 lemma.TR:τα" morph="robinson:T-APN"/><w src="2" lemma="strong:G3588 lemma.TR:τα" morph="robinson:T-APN"/><w src="1" lemma="strong:G1487 lemma.TR:ει" morph="robinson:COND">If</w> <w src="4" lemma="strong:G3004 lemma.TR:ειπον" morph="robinson:V-2AAI-1S">I have told</w> <w src="5" lemma="strong:G4771 lemma.TR:υμιν" morph="robinson:P-2DP">you</w> <w src="3" lemma="strong:G1919 lemma.TR:επιγεια" morph="robinson:A-APN">earthly things</w>, <w src="6" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="8" lemma="strong:G4100 lemma.TR:πιστευετε" morph="robinson:V-PAI-2P">ye believe</w> <w src="7" lemma="strong:G3756 lemma.TR:ου" morph="robinson:PRT-N">not</w>, <w src="9" lemma="strong:G4459 lemma.TR:πως" morph="robinson:ADV-I">how</w> <w src="15" lemma="strong:G4100 lemma.TR:πιστευσετε" morph="robinson:V-FAI-2P">shall ye believe</w>, <w src="10" lemma="strong:G1437 lemma.TR:εαν" morph="robinson:COND">if</w> <w src="11" lemma="strong:G3004 lemma.TR:ειπω" morph="robinson:V-2AAS-1S">I tell</w> <w src="12" lemma="strong:G4771 lemma.TR:υμιν" morph="robinson:P-2DP">you</w> <transChange type="added">of</transChange> <w src="14" lemma="strong:G2032 lemma.TR:επουρανια" morph="robinson:A-APN">heavenly things</w>?<verse eID="John.3.12"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="John.3.12" sID="John.3.12"/><w/><w/><w>If</w> <w>I have told</w> <w>you</w> <w>earthly things</w>, <w>and</w> <w>ye believe</w> <w>not</w>, <w>how</w> <w>shall ye believe</w>, <w>if</w> <w>I tell</w> <w>you</w> <transChange type="added">of</transChange> <w>heavenly things</w>?<verse eID="John.3.12"/>
```

### JHN.3.13 (OSIS `John.3.13`)

- offset 19933976, length 1777, excerpt SHA-256 `cd176fcfcd81af266b0fbc154cb96a24236bad60ea9974cad3945e18b6382866`
- `q` open at verse start: **yes**, its `<q>` tag is in `John.3.10` at byte 19930186
- `q` open at verse end: **yes** (continues into the next verse)
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="John.3.13" sID="John.3.13"/><w src="18" lemma="strong:G3588 lemma.TR:ο" morph="robinson:T-NSM"/><w src="1" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">And</w> <w src="2" lemma="strong:G3762 lemma.TR:ουδεις" morph="robinson:A-NSM-N">no man</w> <w src="3" lemma="strong:G305 lemma.TR:αναβεβηκεν" morph="robinson:V-RAI-3S">hath ascended up</w> <w src="4" lemma="strong:G1519 lemma.TR:εις" morph="robinson:PREP">to</w> <w src="5 6" lemma="strong:G3588 strong:G3772 lemma.TR:τον lemma.TR:ουρανον" morph="robinson:T-ASM robinson:N-ASM">heaven</w>, <w src="7" lemma="strong:G1487 lemma.TR:ει" morph="robinson:COND"/><w src="8" lemma="strong:G3361 lemma.TR:μη" morph="robinson:PRT-N">but</w> <w src="9" lemma="strong:G3588 lemma.TR:ο" morph="robinson:T-NSM">he that</w> <w src="13" lemma="strong:G2597 lemma.TR:καταβας" morph="robinson:V-2AAP-NSM">came down</w> <w src="10" lemma="strong:G1537 lemma.TR:εκ" morph="robinson:PREP">from</w> <w src="11 12" lemma="strong:G3588 strong:G3772 lemma.TR:του lemma.TR:ουρανου" morph="robinson:T-GSM robinson:N-GSM">heaven</w>, <transChange type="added">even</transChange> <w src="14 15" lemma="strong:G3588 strong:G5207 lemma.TR:ο lemma.TR:υιος" morph="robinson:T-NSM robinson:N-NSM">the Son</w> <w src="16 17" lemma="strong:G3588 strong:G444 lemma.TR:του lemma.TR:ανθρωπου" morph="robinson:T-GSM robinson:N-GSM">of man</w> <w src="19" lemma="strong:G1510 lemma.TR:ων" morph="robinson:V-PAP-NSM">which is</w> <w src="20" lemma="strong:G1722 lemma.TR:εν" morph="robinson:PREP">in</w> <w src="21 22" lemma="strong:G3588 strong:G3772 lemma.TR:τω lemma.TR:ουρανω" morph="robinson:T-DSM robinson:N-DSM">heaven</w>.<verse eID="John.3.13"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="John.3.13" sID="John.3.13"/><w/><w>And</w> <w>no man</w> <w>hath ascended up</w> <w>to</w> <w>heaven</w>, <w/><w>but</w> <w>he that</w> <w>came down</w> <w>from</w> <w>heaven</w>, <transChange type="added">even</transChange> <w>the Son</w> <w>of man</w> <w>which is</w> <w>in</w> <w>heaven</w>.<verse eID="John.3.13"/>
```

### JHN.3.14 (OSIS `John.3.14`)

- offset 19935754, length 1355, excerpt SHA-256 `a12b58f6c3491b07f81ea76a2a644fb5e8bbaf35937f72a5200ab9630b9467e1`
- `q` open at verse start: **yes**, its `<q>` tag is in `John.3.10` at byte 19930186
- `q` open at verse end: **yes** (continues into the next verse)
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="John.3.14" sID="John.3.14"/><milestone type="x-p" marker="¶"/><w src="8" lemma="strong:G3588 lemma.TR:τη" morph="robinson:T-DSF"/><w src="1" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">And</w> <w src="2" lemma="strong:G2531 lemma.TR:καθως" morph="robinson:ADV">as</w> <w src="3" lemma="strong:G3475 lemma.TR:μωσης" morph="robinson:N-NSM">Moses</w> <w src="4" lemma="strong:G5312 lemma.TR:υψωσεν" morph="robinson:V-AAI-3S">lifted up</w> <w src="5 6" lemma="strong:G3588 strong:G3789 lemma.TR:τον lemma.TR:οφιν" morph="robinson:T-ASM robinson:N-ASM">the serpent</w> <w src="7" lemma="strong:G1722 lemma.TR:εν" morph="robinson:PREP">in</w> <w src="9" lemma="strong:G2048 lemma.TR:ερημω" morph="robinson:A-DSF">the wilderness</w>, <w src="10" lemma="strong:G3779 lemma.TR:ουτως" morph="robinson:ADV">even so</w> <w src="12" lemma="strong:G1163 lemma.TR:δει" morph="robinson:V-PAI-3S">must</w> <w src="13 14" lemma="strong:G3588 strong:G5207 lemma.TR:τον lemma.TR:υιον" morph="robinson:T-ASM robinson:N-ASM">the Son</w> <w src="15 16" lemma="strong:G3588 strong:G444 lemma.TR:του lemma.TR:ανθρωπου" morph="robinson:T-GSM robinson:N-GSM">of man</w> <w src="11" lemma="strong:G5312 lemma.TR:υψωθηναι" morph="robinson:V-APN">be lifted up</w>:<verse eID="John.3.14"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="John.3.14" sID="John.3.14"/><milestone type="x-p" marker="¶"/><w/><w>And</w> <w>as</w> <w>Moses</w> <w>lifted up</w> <w>the serpent</w> <w>in</w> <w>the wilderness</w>, <w>even so</w> <w>must</w> <w>the Son</w> <w>of man</w> <w>be lifted up</w>:<verse eID="John.3.14"/>
```

### JHN.3.15 (OSIS `John.3.15`)

- offset 19937110, length 1200, excerpt SHA-256 `cddf42a4a904f69a2e5e0f0d3cf3617ad867536cc932868fae7689c0e94aaf95`
- `q` open at verse start: **yes**, its `<q>` tag is in `John.3.10` at byte 19930186
- `q` open at verse end: **yes** (continues into the next verse)
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="John.3.15" sID="John.3.15"/><w src="3" lemma="strong:G3588 lemma.TR:ο" morph="robinson:T-NSM"/><w src="1" lemma="strong:G2443 lemma.TR:ινα" morph="robinson:CONJ">That</w> <w src="2" lemma="strong:G3956 lemma.TR:πας" morph="robinson:A-NSM">whosoever</w> <w src="4" lemma="strong:G4100 lemma.TR:πιστευων" morph="robinson:V-PAP-NSM">believeth</w> <w src="5" lemma="strong:G1519 lemma.TR:εις" morph="robinson:PREP">in</w> <w src="6" lemma="strong:G846 lemma.TR:αυτον" morph="robinson:P-ASM">him</w> <w src="8" lemma="strong:G622 lemma.TR:αποληται" morph="robinson:V-2AMS-3S" type="x-split-1792">should</w> <w src="7" lemma="strong:G3361 lemma.TR:μη" morph="robinson:PRT-N">not</w> <w src="8" lemma="strong:G622 lemma.TR:αποληται" morph="robinson:V-2AMS-3S" type="x-split-1792">perish</w>, <w src="9" lemma="strong:G235 lemma.TR:αλλ" morph="robinson:CONJ">but</w> <w src="10" lemma="strong:G2192 lemma.TR:εχη" morph="robinson:V-PAS-3S">have</w> <w src="12" lemma="strong:G166 lemma.TR:αιωνιον" morph="robinson:A-ASF">eternal</w> <w src="11" lemma="strong:G2222 lemma.TR:ζωην" morph="robinson:N-ASF">life</w>.<verse eID="John.3.15"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="John.3.15" sID="John.3.15"/><w/><w>That</w> <w>whosoever</w> <w>believeth</w> <w>in</w> <w>him</w> <w>should</w> <w>not</w> <w>perish</w>, <w>but</w> <w>have</w> <w>eternal</w> <w>life</w>.<verse eID="John.3.15"/>
```

### JHN.3.16 (OSIS `John.3.16`)

- offset 19938311, length 2319, excerpt SHA-256 `442c8f3e62c870b4194fd238a9d4f5461ed597cee92f662a2486e1b24244204a`
- `q` open at verse start: **yes**, its `<q>` tag is in `John.3.10` at byte 19930186
- `q` open at verse end: **yes** (continues into the next verse)
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="John.3.16" sID="John.3.16"/><milestone type="x-p" marker="¶"/><w src="17" lemma="strong:G3588 lemma.TR:ο" morph="robinson:T-NSM"/><w src="12" lemma="strong:G3588 lemma.TR:τον" morph="robinson:T-ASM"/><w src="2" lemma="strong:G1063 lemma.TR:γαρ" morph="robinson:CONJ">For</w> <w src="4 5" lemma="strong:G3588 strong:G2316 lemma.TR:ο lemma.TR:θεος" morph="robinson:T-NSM robinson:N-NSM">God</w> <w src="1" lemma="strong:G3779 lemma.TR:ουτως" morph="robinson:ADV">so</w> <w src="3" lemma="strong:G25 lemma.TR:ηγαπησεν" morph="robinson:V-AAI-3S">loved</w> <w src="6 7" lemma="strong:G3588 strong:G2889 lemma.TR:τον lemma.TR:κοσμον" morph="robinson:T-ASM robinson:N-ASM">the world</w>, <w src="8" lemma="strong:G5620 lemma.TR:ωστε" morph="robinson:CONJ">that</w> <w src="14" lemma="strong:G1325 lemma.TR:εδωκεν" morph="robinson:V-AAI-3S">he gave</w> <w src="11" lemma="strong:G846 lemma.TR:αυτου" morph="robinson:P-GSM">his</w> <w src="13" lemma="strong:G3439 lemma.TR:μονογενη" morph="robinson:A-ASM">only begotten</w> <w src="9 10" lemma="strong:G3588 strong:G5207 lemma.TR:τον lemma.TR:υιον" morph="robinson:T-ASM robinson:N-ASM">Son</w>, <w src="15" lemma="strong:G2443 lemma.TR:ινα" morph="robinson:CONJ">that</w> <w src="16" lemma="strong:G3956 lemma.TR:πας" morph="robinson:A-NSM">whosoever</w> <w src="18" lemma="strong:G4100 lemma.TR:πιστευων" morph="robinson:V-PAP-NSM">believeth</w> <w src="19" lemma="strong:G1519 lemma.TR:εις" morph="robinson:PREP">in</w> <w src="20" lemma="strong:G846 lemma.TR:αυτον" morph="robinson:P-ASM">him</w> <w src="22" lemma="strong:G622 lemma.TR:αποληται" morph="robinson:V-2AMS-3S" type="x-split-1793">should</w> <w src="21" lemma="strong:G3361 lemma.TR:μη" morph="robinson:PRT-N">not</w> <w src="22" lemma="strong:G622 lemma.TR:αποληται" morph="robinson:V-2AMS-3S" type="x-split-1793">perish</w>, <w src="23" lemma="strong:G235 lemma.TR:αλλ" morph="robinson:CONJ">but</w> <w src="24" lemma="strong:G2192 lemma.TR:εχη" morph="robinson:V-PAS-3S">have</w> <w src="26" lemma="strong:G166 lemma.TR:αιωνιον" morph="robinson:A-ASF">everlasting</w> <w src="25" lemma="strong:G2222 lemma.TR:ζωην" morph="robinson:N-ASF">life</w>.<verse eID="John.3.16"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="John.3.16" sID="John.3.16"/><milestone type="x-p" marker="¶"/><w/><w/><w>For</w> <w>God</w> <w>so</w> <w>loved</w> <w>the world</w>, <w>that</w> <w>he gave</w> <w>his</w> <w>only begotten</w> <w>Son</w>, <w>that</w> <w>whosoever</w> <w>believeth</w> <w>in</w> <w>him</w> <w>should</w> <w>not</w> <w>perish</w>, <w>but</w> <w>have</w> <w>everlasting</w> <w>life</w>.<verse eID="John.3.16"/>
```

### JHN.3.17 (OSIS `John.3.17`)

- offset 19940631, length 1730, excerpt SHA-256 `e9b18251f70de5e375996b84311907551536145d63fd911fd87cddfc733b6c65`
- `q` open at verse start: **yes**, its `<q>` tag is in `John.3.10` at byte 19930186
- `q` open at verse end: **yes** (continues into the next verse)
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="John.3.17" sID="John.3.17"/><w src="2" lemma="strong:G1063 lemma.TR:γαρ" morph="robinson:CONJ">For</w> <w src="4 5" lemma="strong:G3588 strong:G2316 lemma.TR:ο lemma.TR:θεος" morph="robinson:T-NSM robinson:N-NSM">God</w> <w src="3" lemma="strong:G649 lemma.TR:απεστειλεν" morph="robinson:V-AAI-3S">sent</w> <w src="1" lemma="strong:G3756 lemma.TR:ου" morph="robinson:PRT-N">not</w> <w src="8" lemma="strong:G846 lemma.TR:αυτου" morph="robinson:P-GSM">his</w> <w src="6 7" lemma="strong:G3588 strong:G5207 lemma.TR:τον lemma.TR:υιον" morph="robinson:T-ASM robinson:N-ASM">Son</w> <w src="9" lemma="strong:G1519 lemma.TR:εις" morph="robinson:PREP">into</w> <w src="10 11" lemma="strong:G3588 strong:G2889 lemma.TR:τον lemma.TR:κοσμον" morph="robinson:T-ASM robinson:N-ASM">the world</w> <w src="12" lemma="strong:G2443 lemma.TR:ινα" morph="robinson:CONJ">to</w> <w src="13" lemma="strong:G2919 lemma.TR:κρινη" morph="robinson:V-PAS-3S">condemn</w> <w src="14 15" lemma="strong:G3588 strong:G2889 lemma.TR:τον lemma.TR:κοσμον" morph="robinson:T-ASM robinson:N-ASM">the world</w>; <w src="16" lemma="strong:G235 lemma.TR:αλλ" morph="robinson:CONJ">but</w> <w src="17" lemma="strong:G2443 lemma.TR:ινα" morph="robinson:CONJ">that</w> <w src="19 20" lemma="strong:G3588 strong:G2889 lemma.TR:ο lemma.TR:κοσμος" morph="robinson:T-NSM robinson:N-NSM">the world</w> <w src="21" lemma="strong:G1223 lemma.TR:δι" morph="robinson:PREP">through</w> <w src="22" lemma="strong:G846 lemma.TR:αυτου" morph="robinson:P-GSM">him</w> <w src="18" lemma="strong:G4982 lemma.TR:σωθη" morph="robinson:V-APS-3S">might be saved</w>.<verse eID="John.3.17"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="John.3.17" sID="John.3.17"/><w>For</w> <w>God</w> <w>sent</w> <w>not</w> <w>his</w> <w>Son</w> <w>into</w> <w>the world</w> <w>to</w> <w>condemn</w> <w>the world</w>; <w>but</w> <w>that</w> <w>the world</w> <w>through</w> <w>him</w> <w>might be saved</w>.<verse eID="John.3.17"/>
```

### JHN.3.18 (OSIS `John.3.18`)

- offset 19942362, length 2275, excerpt SHA-256 `a580115ec1a7d9675b689ba6cfced7c69e1c89bc67f20570fa190ff1bca660fa`
- `q` open at verse start: **yes**, its `<q>` tag is in `John.3.10` at byte 19930186
- `q` open at verse end: **yes** (continues into the next verse)
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="John.3.18" sID="John.3.18"/><milestone type="x-p" marker="¶"/><w src="19" lemma="strong:G3588 lemma.TR:του" morph="robinson:T-GSM"/><w src="7" lemma="strong:G3588 lemma.TR:ο" morph="robinson:T-NSM"/><w src="1" lemma="strong:G3588 lemma.TR:ο" morph="robinson:T-NSM"/><w src="2" lemma="strong:G4100 lemma.TR:πιστευων" morph="robinson:V-PAP-NSM">He that believeth</w> <w src="3" lemma="strong:G1519 lemma.TR:εις" morph="robinson:PREP">on</w> <w src="4" lemma="strong:G846 lemma.TR:αυτον" morph="robinson:P-ASM">him</w> <w src="6" lemma="strong:G2919 lemma.TR:κρινεται" morph="robinson:V-PPI-3S" type="x-split-1794">is</w> <w src="5" lemma="strong:G3756 lemma.TR:ου" morph="robinson:PRT-N">not</w> <w src="6" lemma="strong:G2919 lemma.TR:κρινεται" morph="robinson:V-PPI-3S" type="x-split-1794">condemned</w>: <w src="8" lemma="strong:G1161 lemma.TR:δε" morph="robinson:CONJ">but</w> <w src="10" lemma="strong:G4100 lemma.TR:πιστευων" morph="robinson:V-PAP-NSM">he that believeth</w> <w src="9" lemma="strong:G3361 lemma.TR:μη" morph="robinson:PRT-N">not</w> <w src="12" lemma="strong:G2919 lemma.TR:κεκριται" morph="robinson:V-RPI-3S">is condemned</w> <w src="11" lemma="strong:G2235 lemma.TR:ηδη" morph="robinson:ADV">already</w>, <w src="13" lemma="strong:G3754 lemma.TR:οτι" morph="robinson:CONJ">because</w> <w src="15" lemma="strong:G4100 lemma.TR:πεπιστευκεν" morph="robinson:V-RAI-3S" type="x-split-1795">he hath</w> <w src="14" lemma="strong:G3361 lemma.TR:μη" morph="robinson:PRT-N">not</w> <w src="15" lemma="strong:G4100 lemma.TR:πεπιστευκεν" morph="robinson:V-RAI-3S" type="x-split-1795">believed</w> <w src="16" lemma="strong:G1519 lemma.TR:εις" morph="robinson:PREP">in</w> <w src="17 18" lemma="strong:G3588 strong:G3686 lemma.TR:το lemma.TR:ονομα" morph="robinson:T-ASN robinson:N-ASN">the name</w> <w src="20" lemma="strong:G3439 lemma.TR:μονογενους" morph="robinson:A-GSM">of the only begotten</w> <w src="21" lemma="strong:G5207 lemma.TR:υιου" morph="robinson:N-GSM">Son</w> <w src="22 23" lemma="strong:G3588 strong:G2316 lemma.TR:του lemma.TR:θεου" morph="robinson:T-GSM robinson:N-GSM">of God</w>.<verse eID="John.3.18"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="John.3.18" sID="John.3.18"/><milestone type="x-p" marker="¶"/><w/><w/><w/><w>He that believeth</w> <w>on</w> <w>him</w> <w>is</w> <w>not</w> <w>condemned</w>: <w>but</w> <w>he that believeth</w> <w>not</w> <w>is condemned</w> <w>already</w>, <w>because</w> <w>he hath</w> <w>not</w> <w>believed</w> <w>in</w> <w>the name</w> <w>of the only begotten</w> <w>Son</w> <w>of God</w>.<verse eID="John.3.18"/>
```

### JHN.3.19 (OSIS `John.3.19`)

- offset 19944638, length 2170, excerpt SHA-256 `17fdf51d07889f416f09b8328b02ed0229c49502a2699888692e68c2424c588e`
- `q` open at verse start: **yes**, its `<q>` tag is in `John.3.10` at byte 19930186
- `q` open at verse end: **yes** (continues into the next verse)
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="John.3.19" sID="John.3.19"/><w src="2" lemma="strong:G1161 lemma.TR:δε" morph="robinson:CONJ">And</w> <w src="1" lemma="strong:G3778 lemma.TR:αυτη" morph="robinson:D-NSF">this</w> <w src="3" lemma="strong:G1510 lemma.TR:εστιν" morph="robinson:V-PAI-3S">is</w> <w src="4 5" lemma="strong:G3588 strong:G2920 lemma.TR:η lemma.TR:κρισις" morph="robinson:T-NSF robinson:N-NSF">the condemnation</w>, <w src="6" lemma="strong:G3754 lemma.TR:οτι" morph="robinson:CONJ">that</w> <w src="7 8" lemma="strong:G3588 strong:G5457 lemma.TR:το lemma.TR:φως" morph="robinson:T-NSN robinson:N-NSN">light</w> <w src="9" lemma="strong:G2064 lemma.TR:εληλυθεν" morph="robinson:V-2RAI-3S">is come</w> <w src="10" lemma="strong:G1519 lemma.TR:εις" morph="robinson:PREP">into</w> <w src="11 12" lemma="strong:G3588 strong:G2889 lemma.TR:τον lemma.TR:κοσμον" morph="robinson:T-ASM robinson:N-ASM">the world</w>, <w src="13" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="15 16" lemma="strong:G3588 strong:G444 lemma.TR:οι lemma.TR:ανθρωποι" morph="robinson:T-NPM robinson:N-NPM">men</w> <w src="14" lemma="strong:G25 lemma.TR:ηγαπησαν" morph="robinson:V-AAI-3P">loved</w> <w src="18 19" lemma="strong:G3588 strong:G4655 lemma.TR:το lemma.TR:σκοτος" morph="robinson:T-ASN robinson:N-ASN">darkness</w> <w src="17" lemma="strong:G3123 lemma.TR:μαλλον" morph="robinson:ADV">rather</w> <w src="20" lemma="strong:G2228 lemma.TR:η" morph="robinson:PRT">than</w> <w src="21 22" lemma="strong:G3588 strong:G5457 lemma.TR:το lemma.TR:φως" morph="robinson:T-ASN robinson:N-ASN">light</w>, <w src="24" lemma="strong:G1063 lemma.TR:γαρ" morph="robinson:CONJ">because</w> <w src="26" lemma="strong:G846 lemma.TR:αυτων" morph="robinson:P-GPM">their</w> <w src="27 28" lemma="strong:G3588 strong:G2041 lemma.TR:τα lemma.TR:εργα" morph="robinson:T-NPN robinson:N-NPN">deeds</w> <w src="23" lemma="strong:G1510 lemma.TR:ην" morph="robinson:V-IAI-3S">were</w> <w src="25" lemma="strong:G4190 lemma.TR:πονηρα" morph="robinson:A-NPN">evil</w>.<verse eID="John.3.19"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="John.3.19" sID="John.3.19"/><w>And</w> <w>this</w> <w>is</w> <w>the condemnation</w>, <w>that</w> <w>light</w> <w>is come</w> <w>into</w> <w>the world</w>, <w>and</w> <w>men</w> <w>loved</w> <w>darkness</w> <w>rather</w> <w>than</w> <w>light</w>, <w>because</w> <w>their</w> <w>deeds</w> <w>were</w> <w>evil</w>.<verse eID="John.3.19"/>
```

### JHN.3.20 (OSIS `John.3.20`)

- offset 19946809, length 1629, excerpt SHA-256 `1503f84c5d87bb03ed901a092ff62b7e715287831a9911438916ff9e993d09ee`
- `q` open at verse start: **yes**, its `<q>` tag is in `John.3.10` at byte 19930186
- `q` open at verse end: **yes** (continues into the next verse)
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="John.3.20" sID="John.3.20"/><w src="3" lemma="strong:G3588 lemma.TR:ο" morph="robinson:T-NSM"/><w src="2" lemma="strong:G1063 lemma.TR:γαρ" morph="robinson:CONJ">For</w> <w src="1" lemma="strong:G3956 lemma.TR:πας" morph="robinson:A-NSM">every</w> <w src="5" lemma="strong:G4238 lemma.TR:πρασσων" morph="robinson:V-PAP-NSM">one that doeth</w> <w src="4" lemma="strong:G5337 lemma.TR:φαυλα" morph="robinson:A-APN">evil</w> <w src="6" lemma="strong:G3404 lemma.TR:μισει" morph="robinson:V-PAI-3S">hateth</w> <w src="7 8" lemma="strong:G3588 strong:G5457 lemma.TR:το lemma.TR:φως" morph="robinson:T-ASN robinson:N-ASN">the light</w>, <w src="9" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ"/><w src="10" lemma="strong:G3756 lemma.TR:ουκ" morph="robinson:PRT-N">neither</w> <w src="11" lemma="strong:G2064 lemma.TR:ερχεται" morph="robinson:V-PNI-3S">cometh</w> <w src="12" lemma="strong:G4314 lemma.TR:προς" morph="robinson:PREP">to</w> <w src="13 14" lemma="strong:G3588 strong:G5457 lemma.TR:το lemma.TR:φως" morph="robinson:T-ASN robinson:N-ASN">the light</w>, <w src="15" lemma="strong:G2443 lemma.TR:ινα" morph="robinson:CONJ"/><w src="16" lemma="strong:G3361 lemma.TR:μη" morph="robinson:PRT-N">lest</w> <w src="20" lemma="strong:G846 lemma.TR:αυτου" morph="robinson:P-GSM">his</w> <w src="18 19" lemma="strong:G3588 strong:G2041 lemma.TR:τα lemma.TR:εργα" morph="robinson:T-NPN robinson:N-NPN">deeds</w> <w src="17" lemma="strong:G1651 lemma.TR:ελεγχθη" morph="robinson:V-APS-3S">should be reproved</w>.<verse eID="John.3.20"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="John.3.20" sID="John.3.20"/><w/><w>For</w> <w>every</w> <w>one that doeth</w> <w>evil</w> <w>hateth</w> <w>the light</w>, <w/><w>neither</w> <w>cometh</w> <w>to</w> <w>the light</w>, <w/><w>lest</w> <w>his</w> <w>deeds</w> <w>should be reproved</w>.<verse eID="John.3.20"/>
```

### JHN.3.21 (OSIS `John.3.21`)

- offset 19948439, length 1583, excerpt SHA-256 `840257216b32e4f7ee00dc20800a35de718aa510333c7e250803c642d7d23645`
- `q` open at verse start: **yes**, its `<q>` tag is in `John.3.10` at byte 19930186
- `q` open at verse end: no
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 1

```xml
<verse osisID="John.3.21" sID="John.3.21"/><w src="1" lemma="strong:G3588 lemma.TR:ο" morph="robinson:T-NSM"/><w src="2" lemma="strong:G1161 lemma.TR:δε" morph="robinson:CONJ">But</w> <w src="3" lemma="strong:G4160 lemma.TR:ποιων" morph="robinson:V-PAP-NSM">he that doeth</w> <w src="4 5" lemma="strong:G3588 strong:G225 lemma.TR:την lemma.TR:αληθειαν" morph="robinson:T-ASF robinson:N-ASF">truth</w> <w src="6" lemma="strong:G2064 lemma.TR:ερχεται" morph="robinson:V-PNI-3S">cometh</w> <w src="7" lemma="strong:G4314 lemma.TR:προς" morph="robinson:PREP">to</w> <w src="8 9" lemma="strong:G3588 strong:G5457 lemma.TR:το lemma.TR:φως" morph="robinson:T-ASN robinson:N-ASN">the light</w>, <w src="10" lemma="strong:G2443 lemma.TR:ινα" morph="robinson:CONJ">that</w> <w src="12" lemma="strong:G846 lemma.TR:αυτου" morph="robinson:P-GSM">his</w> <w src="13 14" lemma="strong:G3588 strong:G2041 lemma.TR:τα lemma.TR:εργα" morph="robinson:T-NPN robinson:N-NPN">deeds</w> <w src="11" lemma="strong:G5319 lemma.TR:φανερωθη" morph="robinson:V-APS-3S">may be made manifest</w>, <w src="15" lemma="strong:G3754 lemma.TR:οτι" morph="robinson:CONJ">that</w> <w src="18" lemma="strong:G1510 lemma.TR:εστιν" morph="robinson:V-PAI-3S">they are</w> <w src="19" lemma="strong:G2038 lemma.TR:ειργασμενα" morph="robinson:V-RPP-NPN">wrought</w> <w src="16" lemma="strong:G1722 lemma.TR:εν" morph="robinson:PREP">in</w> <w src="17" lemma="strong:G2316 lemma.TR:θεω" morph="robinson:N-DSM">God</w>.</q><verse eID="John.3.21"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="John.3.21" sID="John.3.21"/><w/><w>But</w> <w>he that doeth</w> <w>truth</w> <w>cometh</w> <w>to</w> <w>the light</w>, <w>that</w> <w>his</w> <w>deeds</w> <w>may be made manifest</w>, <w>that</w> <w>they are</w> <w>wrought</w> <w>in</w> <w>God</w>.</q><verse eID="John.3.21"/>
```

### JHN.3.22 (OSIS `John.3.22`)

- offset 19950023, length 1748, excerpt SHA-256 `2dda5b0c444e873ca27ffad1c5c7fe35413a26d5984f4955984acef39b4e52de`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="John.3.22" sID="John.3.22"/><milestone type="x-p" subType="x-added" marker="¶"/><w src="11" lemma="strong:G3588 lemma.TR:την" morph="robinson:T-ASF"/><w src="1" lemma="strong:G3326 lemma.TR:μετα" morph="robinson:PREP">After</w> <w src="2" lemma="strong:G3778 lemma.TR:ταυτα" morph="robinson:D-APN">these things</w> <w src="3" lemma="strong:G2064 lemma.TR:ηλθεν" morph="robinson:V-2AAI-3S">came</w> <w src="4 5" lemma="strong:G3588 strong:G2424 lemma.TR:ο lemma.TR:ιησους" morph="robinson:T-NSM robinson:N-NSM">Jesus</w> <w src="6" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="9" lemma="strong:G846 lemma.TR:αυτου" morph="robinson:P-GSM">his</w> <w src="7 8" lemma="strong:G3588 strong:G3101 lemma.TR:οι lemma.TR:μαθηται" morph="robinson:T-NPM robinson:N-NPM">disciples</w> <w src="10" lemma="strong:G1519 lemma.TR:εις" morph="robinson:PREP">into</w> <w src="13" lemma="strong:G1093 lemma.TR:γην" morph="robinson:N-ASF">the land</w> <w src="12" lemma="strong:G2453 lemma.TR:ιουδαιαν" morph="robinson:A-ASF">of Judea</w>; <w src="14" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="15" lemma="strong:G1563 lemma.TR:εκει" morph="robinson:ADV">there</w> <w src="16" lemma="strong:G1304 lemma.TR:διετριβεν" morph="robinson:V-IAI-3S">he tarried</w> <w src="17" lemma="strong:G3326 lemma.TR:μετ" morph="robinson:PREP">with</w> <w src="18" lemma="strong:G846 lemma.TR:αυτων" morph="robinson:P-GPM">them</w>, <w src="19" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="20" lemma="strong:G907 lemma.TR:εβαπτιζεν" morph="robinson:V-IAI-3S">baptized</w>.<verse eID="John.3.22"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="John.3.22" sID="John.3.22"/><milestone type="x-p" subType="x-added" marker="¶"/><w/><w>After</w> <w>these things</w> <w>came</w> <w>Jesus</w> <w>and</w> <w>his</w> <w>disciples</w> <w>into</w> <w>the land</w> <w>of Judea</w>; <w>and</w> <w>there</w> <w>he tarried</w> <w>with</w> <w>them</w>, <w>and</w> <w>baptized</w>.<verse eID="John.3.22"/>
```

### JHN.11.34 (OSIS `John.11.34`)

- offset 20601040, length 1052, excerpt SHA-256 `ab9461ce68c930e5f2684a6c278acda1b4a596b894ab0391724a550f1af6593e`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 1; `</q>` end tags in excerpt: 1

```xml
<verse osisID="John.11.34" sID="John.11.34"/><w src="1" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">And</w> <w src="2" lemma="strong:G3004 lemma.TR:ειπεν" morph="robinson:V-2AAI-3S">said</w>, <q who="Jesus" marker=""><w src="3" lemma="strong:G4226 lemma.TR:που" morph="robinson:ADV-I">Where</w> <w src="4" lemma="strong:G5087 lemma.TR:τεθεικατε" morph="robinson:V-RAI-2P">have ye laid</w> <w src="5" lemma="strong:G846 lemma.TR:αυτον" morph="robinson:P-ASM">him</w>?</q> <w src="6" lemma="strong:G3004 lemma.TR:λεγουσιν" morph="robinson:V-PAI-3P">They said</w> <w src="7" lemma="strong:G846 lemma.TR:αυτω" morph="robinson:P-DSM">unto him</w>, <w src="8" lemma="strong:G2962 lemma.TR:κυριε" morph="robinson:N-VSM">Lord</w>, <w src="9" lemma="strong:G2064 lemma.TR:ερχου" morph="robinson:V-PNM-2S">come</w> <w src="10" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="11" lemma="strong:G3708 lemma.TR:ιδε" morph="robinson:V-2AAM-2S">see</w>.<verse eID="John.11.34"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="John.11.34" sID="John.11.34"/><w>And</w> <w>said</w>, <q who="Jesus" marker=""><w>Where</w> <w>have ye laid</w> <w>him</w>?</q> <w>They said</w> <w>unto him</w>, <w>Lord</w>, <w>come</w> <w>and</w> <w>see</w>.<verse eID="John.11.34"/>
```

### JHN.11.35 (OSIS `John.11.35`)

- offset 20602093, length 294, excerpt SHA-256 `ac0c2c50e353c18e2ec787f3c9a883ddce9efc7660d2637901b7cd4954703546`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="John.11.35" sID="John.11.35"/><w src="2 3" lemma="strong:G3588 strong:G2424 lemma.TR:ο lemma.TR:ιησους" morph="robinson:T-NSM robinson:N-NSM">Jesus</w> <w src="1" lemma="strong:G1145 lemma.TR:εδακρυσεν" morph="robinson:V-AAI-3S">wept</w>.<verse eID="John.11.35"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="John.11.35" sID="John.11.35"/><w>Jesus</w> <w>wept</w>.<verse eID="John.11.35"/>
```

### JHN.19.26 (OSIS `John.19.26`)

- offset 21088598, length 1688, excerpt SHA-256 `a48c6d1f4005734f254b3b2bcdf2ec8f02e641c1506e4233974d2be0d40b83bc`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 1; `</q>` end tags in excerpt: 1

```xml
<verse osisID="John.19.26" sID="John.19.26"/><w src="1" lemma="strong:G2424 lemma.TR:ιησους" morph="robinson:N-NSM">When Jesus</w> <w src="2" lemma="strong:G3767 lemma.TR:ουν" morph="robinson:CONJ">therefore</w> <w src="3" lemma="strong:G3708 lemma.TR:ιδων" morph="robinson:V-2AAP-NSM">saw</w> <w src="4 5" lemma="strong:G3588 strong:G3384 lemma.TR:την lemma.TR:μητερα" morph="robinson:T-ASF robinson:N-ASF">his mother</w>, <w src="6" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="7 8" lemma="strong:G3588 strong:G3101 lemma.TR:τον lemma.TR:μαθητην" morph="robinson:T-ASM robinson:N-ASM">the disciple</w> <w src="9" lemma="strong:G3936 lemma.TR:παρεστωτα" morph="robinson:V-RAP-ASM">standing by</w>, <w src="10" lemma="strong:G3739 lemma.TR:ον" morph="robinson:R-ASM">whom</w> <w src="11" lemma="strong:G25 lemma.TR:ηγαπα" morph="robinson:V-IAI-3S">he loved</w>, <w src="12" lemma="strong:G3004 lemma.TR:λεγει" morph="robinson:V-PAI-3S">he saith</w> <w src="15" lemma="strong:G846 lemma.TR:αυτου" morph="robinson:P-GSM">unto his</w> <w src="13 14" lemma="strong:G3588 strong:G3384 lemma.TR:τη lemma.TR:μητρι" morph="robinson:T-DSF robinson:N-DSF">mother</w>, <q who="Jesus" marker=""><w src="16" lemma="strong:G1135 lemma.TR:γυναι" morph="robinson:N-VSF">Woman</w>, <w src="17" lemma="strong:G3708 lemma.TR:ιδου" morph="robinson:V-2AMM-2S">behold</w> <w src="20" lemma="strong:G4771 lemma.TR:σου" morph="robinson:P-2GS">thy</w> <w src="18 19" lemma="strong:G3588 strong:G5207 lemma.TR:ο lemma.TR:υιος" morph="robinson:T-NSM robinson:N-NSM">son</w>!</q><verse eID="John.19.26"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="John.19.26" sID="John.19.26"/><w>When Jesus</w> <w>therefore</w> <w>saw</w> <w>his mother</w>, <w>and</w> <w>the disciple</w> <w>standing by</w>, <w>whom</w> <w>he loved</w>, <w>he saith</w> <w>unto his</w> <w>mother</w>, <q who="Jesus" marker=""><w>Woman</w>, <w>behold</w> <w>thy</w> <w>son</w>!</q><verse eID="John.19.26"/>
```

### JHN.19.27 (OSIS `John.19.27`)

- offset 21090287, length 1679, excerpt SHA-256 `af4d70c704fff2180edabd744ab1b2a42be50c726a75e8b0de855986b4c312f7`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 1; `</q>` end tags in excerpt: 1

```xml
<verse osisID="John.19.27" sID="John.19.27"/><w src="19" lemma="strong:G3588 lemma.TR:τα" morph="robinson:T-APN"/><w src="1" lemma="strong:G1534 lemma.TR:ειτα" morph="robinson:ADV">Then</w> <w src="2" lemma="strong:G3004 lemma.TR:λεγει" morph="robinson:V-PAI-3S">saith he</w> <w src="3 4" lemma="strong:G3588 strong:G3101 lemma.TR:τω lemma.TR:μαθητη" morph="robinson:T-DSM robinson:N-DSM">to the disciple</w>, <q who="Jesus" marker=""><w src="5" lemma="strong:G3708 lemma.TR:ιδου" morph="robinson:V-2AMM-2S">Behold</w> <w src="8" lemma="strong:G4771 lemma.TR:σου" morph="robinson:P-2GS">thy</w> <w src="6 7" lemma="strong:G3588 strong:G3384 lemma.TR:η lemma.TR:μητηρ" morph="robinson:T-NSF robinson:N-NSF">mother</w>!</q> <w src="9" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">And</w> <w src="10" lemma="strong:G575 lemma.TR:απ" morph="robinson:PREP">from</w> <w src="11" lemma="strong:G1565 lemma.TR:εκεινης" morph="robinson:D-GSF">that</w> <w src="12 13" lemma="strong:G3588 strong:G5610 lemma.TR:της lemma.TR:ωρας" morph="robinson:T-GSF robinson:N-GSF">hour</w> <w src="16 17" lemma="strong:G3588 strong:G3101 lemma.TR:ο lemma.TR:μαθητης" morph="robinson:T-NSM robinson:N-NSM">that disciple</w> <w src="14" lemma="strong:G2983 lemma.TR:ελαβεν" morph="robinson:V-2AAI-3S">took</w> <w src="15" lemma="strong:G846 lemma.TR:αυτην" morph="robinson:P-ASF">her</w> <w src="18" lemma="strong:G1519 lemma.TR:εις" morph="robinson:PREP">unto</w> <w src="20" lemma="strong:G2398 lemma.TR:ιδια" morph="robinson:A-APN">his own</w> <transChange type="added">home</transChange>.<verse eID="John.19.27"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="John.19.27" sID="John.19.27"/><w/><w>Then</w> <w>saith he</w> <w>to the disciple</w>, <q who="Jesus" marker=""><w>Behold</w> <w>thy</w> <w>mother</w>!</q> <w>And</w> <w>from</w> <w>that</w> <w>hour</w> <w>that disciple</w> <w>took</w> <w>her</w> <w>unto</w> <w>his own</w> <transChange type="added">home</transChange>.<verse eID="John.19.27"/>
```

### JHN.19.28 (OSIS `John.19.28`)

- offset 21091967, length 1384, excerpt SHA-256 `27480798fe199a39a59b9249a7d4ff3b498d841db25ea53ba1f4fa717d920cf8`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 1; `</q>` end tags in excerpt: 1

```xml
<verse osisID="John.19.28" sID="John.19.28"/><milestone type="x-p" marker="¶"/><w src="1" lemma="strong:G3326 lemma.TR:μετα" morph="robinson:PREP">After</w> <w src="2" lemma="strong:G3778 lemma.TR:τουτο" morph="robinson:D-ASN">this</w>, <w src="4 5" lemma="strong:G3588 strong:G2424 lemma.TR:ο lemma.TR:ιησους" morph="robinson:T-NSM robinson:N-NSM">Jesus</w> <w src="3" lemma="strong:G1492 lemma.TR:ειδως" morph="robinson:V-RAP-NSM">knowing</w> <w src="6" lemma="strong:G3754 lemma.TR:οτι" morph="robinson:CONJ">that</w> <w src="7" lemma="strong:G3956 lemma.TR:παντα" morph="robinson:A-NPN">all things</w> <w src="8" lemma="strong:G2235 lemma.TR:ηδη" morph="robinson:ADV">were now</w> <w src="9" lemma="strong:G5055 lemma.TR:τετελεσται" morph="robinson:V-RPI-3S">accomplished</w>, <w src="10" lemma="strong:G2443 lemma.TR:ινα" morph="robinson:CONJ">that</w> <w src="12 13" lemma="strong:G3588 strong:G1124 lemma.TR:η lemma.TR:γραφη" morph="robinson:T-NSF robinson:N-NSF">the scripture</w> <w src="11" lemma="strong:G5048 lemma.TR:τελειωθη" morph="robinson:V-APS-3S">might be fulfilled</w>, <w src="14" lemma="strong:G3004 lemma.TR:λεγει" morph="robinson:V-PAI-3S">saith</w>, <q who="Jesus" marker=""><w src="15" lemma="strong:G1372 lemma.TR:διψω" morph="robinson:V-PAI-1S">I thirst</w>.</q><verse eID="John.19.28"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="John.19.28" sID="John.19.28"/><milestone type="x-p" marker="¶"/><w>After</w> <w>this</w>, <w>Jesus</w> <w>knowing</w> <w>that</w> <w>all things</w> <w>were now</w> <w>accomplished</w>, <w>that</w> <w>the scripture</w> <w>might be fulfilled</w>, <w>saith</w>, <q who="Jesus" marker=""><w>I thirst</w>.</q><verse eID="John.19.28"/>
```

### JHN.19.30 (OSIS `John.19.30`)

- offset 21094984, length 1377, excerpt SHA-256 `943931162fb3da563cbe18385c5d0b88a148eb9c2fd05687704c5ab2a494ac5a`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 1; `</q>` end tags in excerpt: 1

```xml
<verse osisID="John.19.30" sID="John.19.30"/><w src="1" lemma="strong:G3753 lemma.TR:οτε" morph="robinson:ADV">When</w> <w src="6 7" lemma="strong:G3588 strong:G2424 lemma.TR:ο lemma.TR:ιησους" morph="robinson:T-NSM robinson:N-NSM">Jesus</w> <w src="2" lemma="strong:G3767 lemma.TR:ουν" morph="robinson:CONJ">therefore</w> <w src="3" lemma="strong:G2983 lemma.TR:ελαβεν" morph="robinson:V-2AAI-3S">had received</w> <w src="4 5" lemma="strong:G3588 strong:G3690 lemma.TR:το lemma.TR:οξος" morph="robinson:T-ASN robinson:N-ASN">the vinegar</w>, <w src="8" lemma="strong:G3004 lemma.TR:ειπεν" morph="robinson:V-2AAI-3S">he said</w>, <q who="Jesus" marker=""><w src="9" lemma="strong:G5055 lemma.TR:τετελεσται" morph="robinson:V-RPI-3S">It is finished</w>:</q> <w src="10" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="11" lemma="strong:G2827 lemma.TR:κλινας" morph="robinson:V-AAP-NSM">he bowed</w> <w src="12 13" lemma="strong:G3588 strong:G2776 lemma.TR:την lemma.TR:κεφαλην" morph="robinson:T-ASF robinson:N-ASF">his head</w>, <w src="14" lemma="strong:G3860 lemma.TR:παρεδωκεν" morph="robinson:V-AAI-3S">and gave up</w> <w src="15 16" lemma="strong:G3588 strong:G4151 lemma.TR:το lemma.TR:πνευμα" morph="robinson:T-ASN robinson:N-ASN">the ghost</w>.<verse eID="John.19.30"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="John.19.30" sID="John.19.30"/><w>When</w> <w>Jesus</w> <w>therefore</w> <w>had received</w> <w>the vinegar</w>, <w>he said</w>, <q who="Jesus" marker=""><w>It is finished</w>:</q> <w>and</w> <w>he bowed</w> <w>his head</w>, <w>and gave up</w> <w>the ghost</w>.<verse eID="John.19.30"/>
```

### ACT.1.4 (OSIS `Acts.1.4`)

- offset 21226775, length 1998, excerpt SHA-256 `5ca937110e5e0a24891ee610bbda7fdfbaabe1e15092e51568de83c8fe45afd8`
- `q` open at verse start: no
- `q` open at verse end: **yes** (continues into the next verse)
- `<q>` start tags in excerpt: 2; `</q>` end tags in excerpt: 1

```xml
<verse osisID="Acts.1.4" sID="Acts.1.4"/><w src="1" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">And</w>, <w src="2" lemma="strong:G4871 lemma.TR:συναλιζομενος" morph="robinson:V-PNP-NSM">being assembled together</w> <w src="3" lemma="strong:G3326 lemma.TR:μετ" morph="robinson:PREP">with</w> <transChange type="added">them</transChange>, <w src="4" lemma="strong:G846 lemma.TR:αυτων" morph="robinson:P-GPM"/><w src="5" lemma="strong:G3853 lemma.TR:παρηγγειλεν" morph="robinson:V-AAI-3S">commanded</w> <w src="6" lemma="strong:G846 lemma.TR:αυτοις" morph="robinson:P-DPM">them</w> <w src="10" lemma="strong:G5563 lemma.TR:χωριζεσθαι" morph="robinson:V-PPN" type="x-split-2076">that they should</w> <w src="9" lemma="strong:G3361 lemma.TR:μη" morph="robinson:PRT-N">not</w> <w src="10" lemma="strong:G5563 lemma.TR:χωριζεσθαι" morph="robinson:V-PPN" type="x-split-2076">depart</w> <w src="7" lemma="strong:G575 lemma.TR:απο" morph="robinson:PREP">from</w> <w src="8" lemma="strong:G2414 lemma.TR:ιεροσολυμων" morph="robinson:N-GPN">Jerusalem</w>, <w src="11" lemma="strong:G235 lemma.TR:αλλα" morph="robinson:CONJ">but</w> <q who="Jesus" marker=""><w src="12" lemma="strong:G4037 lemma.TR:περιμενειν" morph="robinson:V-PAN">wait for</w> <w src="13 14" lemma="strong:G3588 strong:G1860 lemma.TR:την lemma.TR:επαγγελιαν" morph="robinson:T-ASF robinson:N-ASF">the promise</w> <w src="15 16" lemma="strong:G3588 strong:G3962 lemma.TR:του lemma.TR:πατρος" morph="robinson:T-GSM robinson:N-GSM">of the Father</w>, <w src="17" lemma="strong:G3739 lemma.TR:ην" morph="robinson:R-ASF">which</w>,</q> <transChange type="added">saith he</transChange>, <q who="Jesus" marker=""><w src="18" lemma="strong:G191 lemma.TR:ηκουσατε" morph="robinson:V-AAI-2P">ye have heard</w> <w src="19" lemma="strong:G1473 lemma.TR:μου" morph="robinson:P-1GS">of me</w>.<verse eID="Acts.1.4"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Acts.1.4" sID="Acts.1.4"/><w>And</w>, <w>being assembled together</w> <w>with</w> <transChange type="added">them</transChange>, <w/><w>commanded</w> <w>them</w> <w>that they should</w> <w>not</w> <w>depart</w> <w>from</w> <w>Jerusalem</w>, <w>but</w> <q who="Jesus" marker=""><w>wait for</w> <w>the promise</w> <w>of the Father</w>, <w>which</w>,</q> <transChange type="added">saith he</transChange>, <q who="Jesus" marker=""><w>ye have heard</w> <w>of me</w>.<verse eID="Acts.1.4"/>
```

### ACT.1.5 (OSIS `Acts.1.5`)

- offset 21228774, length 1569, excerpt SHA-256 `9a85e0bd04728eca5ec265b04b94ac7d05f09fe7a83ec9ecc9df9de6c6d6ce18`
- `q` open at verse start: **yes**, its `<q>` tag is in `Acts.1.4` at byte 21228541
- `q` open at verse end: no
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 1

```xml
<verse osisID="Acts.1.5" sID="Acts.1.5"/><w src="1" lemma="strong:G3754 lemma.TR:οτι" morph="robinson:CONJ">For</w> <w src="2" lemma="strong:G2491 lemma.TR:ιωαννης" morph="robinson:N-NSM">John</w> <w src="3" lemma="strong:G3303 lemma.TR:μεν" morph="robinson:PRT">truly</w> <w src="4" lemma="strong:G907 lemma.TR:εβαπτισεν" morph="robinson:V-AAI-3S">baptized</w> <w src="5" lemma="strong:G5204 lemma.TR:υδατι" morph="robinson:N-DSN">with water</w>; <w src="7" lemma="strong:G1161 lemma.TR:δε" morph="robinson:CONJ">but</w> <w src="6" lemma="strong:G4771 lemma.TR:υμεις" morph="robinson:P-2NP">ye</w> <w src="8" lemma="strong:G907 lemma.TR:βαπτισθησεσθε" morph="robinson:V-FPI-2P">shall be baptized</w> <w src="9" lemma="strong:G1722 lemma.TR:εν" morph="robinson:PREP">with</w> <w src="10" lemma="strong:G4151 lemma.TR:πνευματι" morph="robinson:N-DSN" type="x-split-2077">the</w> <w src="11" lemma="strong:G40 lemma.TR:αγιω" morph="robinson:A-DSN">Holy</w> <w src="10" lemma="strong:G4151 lemma.TR:πνευματι" morph="robinson:N-DSN" type="x-split-2077">Ghost</w> <w src="12" lemma="strong:G3756 lemma.TR:ου" morph="robinson:PRT-N">not</w> <w src="14" lemma="strong:G4183 lemma.TR:πολλας" morph="robinson:A-APF">many</w> <w src="16" lemma="strong:G2250 lemma.TR:ημερας" morph="robinson:N-APF">days</w> <w src="13" lemma="strong:G3326 lemma.TR:μετα" morph="robinson:PREP">hence</w>.<w src="15" lemma="strong:G3778 lemma.TR:ταυτας" morph="robinson:D-APF"/></q><verse eID="Acts.1.5"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Acts.1.5" sID="Acts.1.5"/><w>For</w> <w>John</w> <w>truly</w> <w>baptized</w> <w>with water</w>; <w>but</w> <w>ye</w> <w>shall be baptized</w> <w>with</w> <w>the</w> <w>Holy</w> <w>Ghost</w> <w>not</w> <w>many</w> <w>days</w> <w>hence</w>.<w/></q><verse eID="Acts.1.5"/>
```

### ACT.1.6 (OSIS `Acts.1.6`)

- offset 21230344, length 1853, excerpt SHA-256 `642aa093f8a16ee411463e44bd6e58fb1276c216ad7224fb6e79bd9def52d391`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="Acts.1.6" sID="Acts.1.6"/><w src="17" lemma="strong:G3588 lemma.TR:τω" morph="robinson:T-DSM"/><w src="2" lemma="strong:G3303 lemma.TR:μεν" morph="robinson:PRT"/><w src="1" lemma="strong:G3588 lemma.TR:οι" morph="robinson:T-NPM"/><w src="4" lemma="strong:G4905 lemma.TR:συνελθοντες" morph="robinson:V-2AAP-NPM" type="x-split-2078">When they</w> <w src="3" lemma="strong:G3767 lemma.TR:ουν" morph="robinson:CONJ">therefore</w> <w src="4" lemma="strong:G4905 lemma.TR:συνελθοντες" morph="robinson:V-2AAP-NPM" type="x-split-2078">were come together</w>, <w src="5" lemma="strong:G1905 lemma.TR:επηρωτων" morph="robinson:V-IAI-3P">they asked</w> <w src="6" lemma="strong:G846 lemma.TR:αυτον" morph="robinson:P-ASM">of him</w>, <w src="7" lemma="strong:G3004 lemma.TR:λεγοντες" morph="robinson:V-PAP-NPM">saying</w>, <w src="8" lemma="strong:G2962 lemma.TR:κυριε" morph="robinson:N-VSM">Lord</w>, <w src="9" lemma="strong:G1487 lemma.TR:ει" morph="robinson:COND">wilt</w> <w src="14" lemma="strong:G600 lemma.TR:αποκαθιστανεις" morph="robinson:V-PAI-2S" type="x-split-2079">thou</w> <w src="10" lemma="strong:G1722 lemma.TR:εν" morph="robinson:PREP">at</w> <w src="13" lemma="strong:G3778 lemma.TR:τουτω" morph="robinson:D-DSM">this</w> <w src="11 12" lemma="strong:G3588 strong:G5550 lemma.TR:τω lemma.TR:χρονω" morph="robinson:T-DSM robinson:N-DSM">time</w> <w src="14" lemma="strong:G600 lemma.TR:αποκαθιστανεις" morph="robinson:V-PAI-2S" type="x-split-2079">restore again</w> <w src="15 16" lemma="strong:G3588 strong:G932 lemma.TR:την lemma.TR:βασιλειαν" morph="robinson:T-ASF robinson:N-ASF">the kingdom</w> <w src="18" lemma="strong:G2474 lemma.TR:ισραηλ" morph="robinson:N-PRI">to Israel</w>?<verse eID="Acts.1.6"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Acts.1.6" sID="Acts.1.6"/><w/><w/><w/><w>When they</w> <w>therefore</w> <w>were come together</w>, <w>they asked</w> <w>of him</w>, <w>saying</w>, <w>Lord</w>, <w>wilt</w> <w>thou</w> <w>at</w> <w>this</w> <w>time</w> <w>restore again</w> <w>the kingdom</w> <w>to Israel</w>?<verse eID="Acts.1.6"/>
```

### ACT.1.7 (OSIS `Acts.1.7`)

- offset 21232198, length 1782, excerpt SHA-256 `76386168f09e843d68ae21514b9528054e6e0eb94e52321180724dfe11a9cc52`
- `q` open at verse start: no
- `q` open at verse end: **yes** (continues into the next verse)
- `<q>` start tags in excerpt: 1; `</q>` end tags in excerpt: 0

```xml
<verse osisID="Acts.1.7" sID="Acts.1.7"/><w src="17" lemma="strong:G3588 lemma.TR:τη" morph="robinson:T-DSF"/><w src="2" lemma="strong:G1161 lemma.TR:δε" morph="robinson:CONJ">And</w> <w src="1" lemma="strong:G3004 lemma.TR:ειπεν" morph="robinson:V-2AAI-3S">he said</w> <w src="3" lemma="strong:G4314 lemma.TR:προς" morph="robinson:PREP">unto</w> <w src="4" lemma="strong:G846 lemma.TR:αυτους" morph="robinson:P-APM">them</w>, <q who="Jesus" marker=""><w src="8" lemma="strong:G1097 lemma.TR:γνωναι" morph="robinson:V-2AAN" type="x-split-2080">It</w> <w src="7" lemma="strong:G1510 lemma.TR:εστιν" morph="robinson:V-PAI-3S">is</w> <w src="5" lemma="strong:G3756 lemma.TR:ουχ" morph="robinson:PRT-N">not</w> <w src="6" lemma="strong:G4771 lemma.TR:υμων" morph="robinson:P-2GP">for you</w> <w src="8" lemma="strong:G1097 lemma.TR:γνωναι" morph="robinson:V-2AAN" type="x-split-2080">to know</w> <w src="9" lemma="strong:G5550 lemma.TR:χρονους" morph="robinson:N-APM">the times</w> <w src="10" lemma="strong:G2228 lemma.TR:η" morph="robinson:PRT">or</w> <w src="11" lemma="strong:G2540 lemma.TR:καιρους" morph="robinson:N-APM">the seasons</w>, <w src="12" lemma="strong:G3739 lemma.TR:ους" morph="robinson:R-APM">which</w> <w src="13 14" lemma="strong:G3588 strong:G3962 lemma.TR:ο lemma.TR:πατηρ" morph="robinson:T-NSM robinson:N-NSM">the Father</w> <w src="15" lemma="strong:G5087 lemma.TR:εθετο" morph="robinson:V-2AMI-3S">hath put</w> <w src="16" lemma="strong:G1722 lemma.TR:εν" morph="robinson:PREP">in</w> <w src="18" lemma="strong:G2398 lemma.TR:ιδια" morph="robinson:A-DSF">his own</w> <w src="19" lemma="strong:G1849 lemma.TR:εξουσια" morph="robinson:N-DSF">power</w>.<verse eID="Acts.1.7"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Acts.1.7" sID="Acts.1.7"/><w/><w>And</w> <w>he said</w> <w>unto</w> <w>them</w>, <q who="Jesus" marker=""><w>It</w> <w>is</w> <w>not</w> <w>for you</w> <w>to know</w> <w>the times</w> <w>or</w> <w>the seasons</w>, <w>which</w> <w>the Father</w> <w>hath put</w> <w>in</w> <w>his own</w> <w>power</w>.<verse eID="Acts.1.7"/>
```

### ACT.1.8 (OSIS `Acts.1.8`)

- offset 21233981, length 2683, excerpt SHA-256 `f892a725a9a2de2a4e5e60b95e660e88c428416b924f1ad63fc1faed0366401b`
- `q` open at verse start: **yes**, its `<q>` tag is in `Acts.1.7` at byte 21232644
- `q` open at verse end: no
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 1

```xml
<verse osisID="Acts.1.8" sID="Acts.1.8"/><w src="5" lemma="strong:G3588 lemma.TR:του" morph="robinson:T-GSN"/><w src="1" lemma="strong:G235 lemma.TR:αλλα" morph="robinson:CONJ">But</w> <w src="2" lemma="strong:G2983 lemma.TR:ληψεσθε" morph="robinson:V-FDI-2P">ye shall receive</w> <w src="3" lemma="strong:G1411 lemma.TR:δυναμιν" morph="robinson:N-ASF">power</w>, <w src="4" lemma="strong:G1904 lemma.TR:επελθοντος" morph="robinson:V-2AAP-GSN" type="x-split-2081">after that</w> <w src="7" lemma="strong:G4151 lemma.TR:πνευματος" morph="robinson:N-GSN" type="x-split-2082">the</w> <w src="6" lemma="strong:G40 lemma.TR:αγιου" morph="robinson:A-GSN">Holy</w> <w src="7" lemma="strong:G4151 lemma.TR:πνευματος" morph="robinson:N-GSN" type="x-split-2082">Ghost</w> <w src="4" lemma="strong:G1904 lemma.TR:επελθοντος" morph="robinson:V-2AAP-GSN" type="x-split-2081">is come</w> <w src="8" lemma="strong:G1909 lemma.TR:εφ" morph="robinson:PREP">upon</w> <w src="9" lemma="strong:G4771 lemma.TR:υμας" morph="robinson:P-2AP">you</w>: <w src="10" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="11" lemma="strong:G1510 lemma.TR:εσεσθε" morph="robinson:V-FDI-2P">ye shall be</w> <w src="13" lemma="strong:G3144 lemma.TR:μαρτυρες" morph="robinson:N-NPM">witnesses</w> <w src="12" lemma="strong:G1473 lemma.TR:μοι" morph="robinson:P-1DS">unto me</w> <w src="15" lemma="strong:G5037 lemma.TR:τε" morph="robinson:PRT">both</w> <w src="14" lemma="strong:G1722 lemma.TR:εν" morph="robinson:PREP">in</w> <w src="16" lemma="strong:G2419 lemma.TR:ιερουσαλημ" morph="robinson:N-PRI">Jerusalem</w>, <w src="17" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="18" lemma="strong:G1722 lemma.TR:εν" morph="robinson:PREP">in</w> <w src="19" lemma="strong:G3956 lemma.TR:παση" morph="robinson:A-DSF">all</w> <w src="20 21" lemma="strong:G3588 strong:G2449 lemma.TR:τη lemma.TR:ιουδαια" morph="robinson:T-DSF robinson:N-DSF">Judea</w>, <w src="22" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="23" lemma="strong:G4540 lemma.TR:σαμαρεια" morph="robinson:N-DSF">in Samaria</w>, <w src="24" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="25" lemma="strong:G2193 lemma.TR:εως" morph="robinson:ADV">unto</w> <w src="26" lemma="strong:G2078 lemma.TR:εσχατου" morph="robinson:A-GSM-S">the uttermost</w> <w src="27 28" lemma="strong:G3588 strong:G1093 lemma.TR:της lemma.TR:γης" morph="robinson:T-GSF robinson:N-GSF">part of the earth</w>.</q><verse eID="Acts.1.8"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Acts.1.8" sID="Acts.1.8"/><w/><w>But</w> <w>ye shall receive</w> <w>power</w>, <w>after that</w> <w>the</w> <w>Holy</w> <w>Ghost</w> <w>is come</w> <w>upon</w> <w>you</w>: <w>and</w> <w>ye shall be</w> <w>witnesses</w> <w>unto me</w> <w>both</w> <w>in</w> <w>Jerusalem</w>, <w>and</w> <w>in</w> <w>all</w> <w>Judea</w>, <w>and</w> <w>in Samaria</w>, <w>and</w> <w>unto</w> <w>the uttermost</w> <w>part of the earth</w>.</q><verse eID="Acts.1.8"/>
```

### ACT.1.9 (OSIS `Acts.1.9`)

- offset 21236665, length 1279, excerpt SHA-256 `f419f179332e53d7b78142a9a1b6072d268f5ac93513a1bb7ef4dcb1c38a6e11`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="Acts.1.9" sID="Acts.1.9"/><w src="1" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">And</w> <w src="3" lemma="strong:G3004 lemma.TR:ειπων" morph="robinson:V-2AAP-NSM">when he had spoken</w> <w src="2" lemma="strong:G3778 lemma.TR:ταυτα" morph="robinson:D-APN">these things</w>, <w src="4" lemma="strong:G991 lemma.TR:βλεποντων" morph="robinson:V-PAP-GPM">while they beheld</w>, <w src="5" lemma="strong:G846 lemma.TR:αυτων" morph="robinson:P-GPM">he</w> <w src="6" lemma="strong:G1869 lemma.TR:επηρθη" morph="robinson:V-API-3S">was taken up</w>; <w src="7" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="8" lemma="strong:G3507 lemma.TR:νεφελη" morph="robinson:N-NSF">a cloud</w> <w src="9" lemma="strong:G5274 lemma.TR:υπελαβεν" morph="robinson:V-2AAI-3S">received</w> <w src="10" lemma="strong:G846 lemma.TR:αυτον" morph="robinson:P-ASM">him</w> <w src="11" lemma="strong:G575 lemma.TR:απο" morph="robinson:PREP">out of</w> <w src="14" lemma="strong:G846 lemma.TR:αυτων" morph="robinson:P-GPM">their</w> <w src="12 13" lemma="strong:G3588 strong:G3788 lemma.TR:των lemma.TR:οφθαλμων" morph="robinson:T-GPM robinson:N-GPM">sight</w>.<verse eID="Acts.1.9"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Acts.1.9" sID="Acts.1.9"/><w>And</w> <w>when he had spoken</w> <w>these things</w>, <w>while they beheld</w>, <w>he</w> <w>was taken up</w>; <w>and</w> <w>a cloud</w> <w>received</w> <w>him</w> <w>out of</w> <w>their</w> <w>sight</w>.<verse eID="Acts.1.9"/>
```

### ACT.1.10 (OSIS `Acts.1.10`)

- offset 21237945, length 1731, excerpt SHA-256 `88c2b58fd7e4c71f50511fda34d8b8c5c5e6c5f2734de248c60cc78f4ae5fe84`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="Acts.1.10" sID="Acts.1.10"/><w src="1" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">And</w> <w src="2" lemma="strong:G5613 lemma.TR:ως" morph="robinson:ADV">while</w> <w src="4" lemma="strong:G1510 lemma.TR:ησαν" morph="robinson:V-IAI-3P"/><w src="3" lemma="strong:G816 lemma.TR:ατενιζοντες" morph="robinson:V-PAP-NPM">they looked stedfastly</w> <w src="5" lemma="strong:G1519 lemma.TR:εις" morph="robinson:PREP">toward</w> <w src="6 7" lemma="strong:G3588 strong:G3772 lemma.TR:τον lemma.TR:ουρανον" morph="robinson:T-ASM robinson:N-ASM">heaven</w> <w src="8" lemma="strong:G4198 lemma.TR:πορευομενου" morph="robinson:V-PNP-GSM" type="x-split-2083">as</w> <w src="9" lemma="strong:G846 lemma.TR:αυτου" morph="robinson:P-GSM">he</w> <w src="8" lemma="strong:G4198 lemma.TR:πορευομενου" morph="robinson:V-PNP-GSM" type="x-split-2083">went up</w>, <w src="10" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ"/><w src="11" lemma="strong:G3708 lemma.TR:ιδου" morph="robinson:V-2AMM-2S">behold</w>, <w src="13" lemma="strong:G1417 lemma.TR:δυο" morph="robinson:A-NUI">two</w> <w src="12" lemma="strong:G435 lemma.TR:ανδρες" morph="robinson:N-NPM">men</w> <w src="14" lemma="strong:G3936 lemma.TR:παρειστηκεισαν" morph="robinson:V-LAI-3P">stood by</w> <w src="15" lemma="strong:G846 lemma.TR:αυτοις" morph="robinson:P-DPM">them</w> <w src="16" lemma="strong:G1722 lemma.TR:εν" morph="robinson:PREP">in</w> <w src="18" lemma="strong:G3022 lemma.TR:λευκη" morph="robinson:A-DSF">white</w> <w src="17" lemma="strong:G2066 lemma.TR:εσθητι" morph="robinson:N-DSF">apparel</w>;<verse eID="Acts.1.10"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Acts.1.10" sID="Acts.1.10"/><w>And</w> <w>while</w> <w/><w>they looked stedfastly</w> <w>toward</w> <w>heaven</w> <w>as</w> <w>he</w> <w>went up</w>, <w/><w>behold</w>, <w>two</w> <w>men</w> <w>stood by</w> <w>them</w> <w>in</w> <w>white</w> <w>apparel</w>;<verse eID="Acts.1.10"/>
```

### ACT.1.11 (OSIS `Acts.1.11`)

- offset 21239677, length 2769, excerpt SHA-256 `2e0b52c449276d1b65be8a5224755abf1baff31c3e89b3ff7d6a99a37ba0746e`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="Acts.1.11" sID="Acts.1.11"/><w src="15" lemma="strong:G3588 lemma.TR:ο" morph="robinson:T-NSM"/><w src="1" lemma="strong:G3739 lemma.TR:οι" morph="robinson:R-NPM">Which</w> <w src="2" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">also</w> <w src="3" lemma="strong:G3004 lemma.TR:ειπον" morph="robinson:V-2AAI-3P">said</w>, <w src="4" lemma="strong:G435 lemma.TR:ανδρες" morph="robinson:N-VPM">Ye men</w> <w src="5" lemma="strong:G1057 lemma.TR:γαλιλαιοι" morph="robinson:N-VPM">of Galilee</w>, <w src="6" lemma="strong:G5101 lemma.TR:τι" morph="robinson:I-ASN">why</w> <w src="7" lemma="strong:G2476 lemma.TR:εστηκατε" morph="robinson:V-RAI-2P">stand ye</w> <w src="8" lemma="strong:G1689 lemma.TR:εμβλεποντες" morph="robinson:V-PAP-NPM">gazing up</w> <w src="9" lemma="strong:G1519 lemma.TR:εις" morph="robinson:PREP">into</w> <w src="10 11" lemma="strong:G3588 strong:G3772 lemma.TR:τον lemma.TR:ουρανον" morph="robinson:T-ASM robinson:N-ASM">heaven</w>? <w src="12" lemma="strong:G3778 lemma.TR:ουτος" morph="robinson:D-NSM">this same</w> <w src="13 14" lemma="strong:G3588 strong:G2424 lemma.TR:ο lemma.TR:ιησους" morph="robinson:T-NSM robinson:N-NSM">Jesus</w>, <w src="16" lemma="strong:G353 lemma.TR:αναληφθεις" morph="robinson:V-APP-NSM">which is taken up</w> <w src="17" lemma="strong:G575 lemma.TR:αφ" morph="robinson:PREP">from</w> <w src="18" lemma="strong:G4771 lemma.TR:υμων" morph="robinson:P-2GP">you</w> <w src="19" lemma="strong:G1519 lemma.TR:εις" morph="robinson:PREP">into</w> <w src="20 21" lemma="strong:G3588 strong:G3772 lemma.TR:τον lemma.TR:ουρανον" morph="robinson:T-ASM robinson:N-ASM">heaven</w>, <w src="23" lemma="strong:G2064 lemma.TR:ελευσεται" morph="robinson:V-FDI-3S" type="x-split-2084">shall</w> <w src="22" lemma="strong:G3779 lemma.TR:ουτως" morph="robinson:ADV">so</w> <w src="23" lemma="strong:G2064 lemma.TR:ελευσεται" morph="robinson:V-FDI-3S" type="x-split-2084">come</w> <w src="25" lemma="strong:G5158 lemma.TR:τροπον" morph="robinson:N-ASM">in like manner</w> <w src="24" lemma="strong:G3739 lemma.TR:ον" morph="robinson:R-ASM">as</w> <w src="26" lemma="strong:G2300 lemma.TR:εθεασασθε" morph="robinson:V-ADI-2P">ye have seen</w> <w src="27" lemma="strong:G846 lemma.TR:αυτον" morph="robinson:P-ASM">him</w> <w src="28" lemma="strong:G4198 lemma.TR:πορευομενον" morph="robinson:V-PNP-ASM">go</w> <w src="29" lemma="strong:G1519 lemma.TR:εις" morph="robinson:PREP">into</w> <w src="30 31" lemma="strong:G3588 strong:G3772 lemma.TR:τον lemma.TR:ουρανον" morph="robinson:T-ASM robinson:N-ASM">heaven</w>.<verse eID="Acts.1.11"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Acts.1.11" sID="Acts.1.11"/><w/><w>Which</w> <w>also</w> <w>said</w>, <w>Ye men</w> <w>of Galilee</w>, <w>why</w> <w>stand ye</w> <w>gazing up</w> <w>into</w> <w>heaven</w>? <w>this same</w> <w>Jesus</w>, <w>which is taken up</w> <w>from</w> <w>you</w> <w>into</w> <w>heaven</w>, <w>shall</w> <w>so</w> <w>come</w> <w>in like manner</w> <w>as</w> <w>ye have seen</w> <w>him</w> <w>go</w> <w>into</w> <w>heaven</w>.<verse eID="Acts.1.11"/>
```

### ACT.9.4 (OSIS `Acts.9.4`)

- offset 21732470, length 1267, excerpt SHA-256 `b71854184c123166b980489ca9d1daa647119830798f5b978f3bd2617850d7c8`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 1; `</q>` end tags in excerpt: 1

```xml
<verse osisID="Acts.9.4" sID="Acts.9.4"/><w src="1" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">And</w> <w src="2" lemma="strong:G4098 lemma.TR:πεσων" morph="robinson:V-2AAP-NSM">he fell</w> <w src="3" lemma="strong:G1909 lemma.TR:επι" morph="robinson:PREP">to</w> <w src="4 5" lemma="strong:G3588 strong:G1093 lemma.TR:την lemma.TR:γην" morph="robinson:T-ASF robinson:N-ASF">the earth</w>, <w src="6" lemma="strong:G191 lemma.TR:ηκουσεν" morph="robinson:V-AAI-3S">and heard</w> <w src="7" lemma="strong:G5456 lemma.TR:φωνην" morph="robinson:N-ASF">a voice</w> <w src="8" lemma="strong:G3004 lemma.TR:λεγουσαν" morph="robinson:V-PAP-ASF">saying</w> <w src="9" lemma="strong:G846 lemma.TR:αυτω" morph="robinson:P-DSM">unto him</w>, <q who="Jesus" marker=""><w src="10" lemma="strong:G4549 lemma.TR:σαουλ" morph="robinson:N-PRI">Saul</w>, <w src="11" lemma="strong:G4549 lemma.TR:σαουλ" morph="robinson:N-PRI">Saul</w>, <w src="12" lemma="strong:G5101 lemma.TR:τι" morph="robinson:I-ASN">why</w> <w src="14" lemma="strong:G1377 lemma.TR:διωκεις" morph="robinson:V-PAI-2S">persecutest thou</w> <w src="13" lemma="strong:G1473 lemma.TR:με" morph="robinson:P-1AS">me</w>?</q><verse eID="Acts.9.4"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Acts.9.4" sID="Acts.9.4"/><w>And</w> <w>he fell</w> <w>to</w> <w>the earth</w>, <w>and heard</w> <w>a voice</w> <w>saying</w> <w>unto him</w>, <q who="Jesus" marker=""><w>Saul</w>, <w>Saul</w>, <w>why</w> <w>persecutest thou</w> <w>me</w>?</q><verse eID="Acts.9.4"/>
```

### ACT.9.5 (OSIS `Acts.9.5`)

- offset 21733738, length 1836, excerpt SHA-256 `c735e7d58442d6400869bdb7a284987aca066454b4862a04a08abc1c4f7ddc6f`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 1; `</q>` end tags in excerpt: 1

```xml
<verse osisID="Acts.9.5" sID="Acts.9.5"/><w src="6" lemma="strong:G3588 lemma.TR:ο" morph="robinson:T-NSM"/><w src="2" lemma="strong:G1161 lemma.TR:δε" morph="robinson:CONJ">And</w> <w src="1" lemma="strong:G3004 lemma.TR:ειπεν" morph="robinson:V-2AAI-3S">he said</w>, <w src="3" lemma="strong:G5101 lemma.TR:τις" morph="robinson:I-NSM">Who</w> <w src="4" lemma="strong:G1510 lemma.TR:ει" morph="robinson:V-PAI-2S">art thou</w>, <w src="5" lemma="strong:G2962 lemma.TR:κυριε" morph="robinson:N-VSM">Lord</w>? <w src="7" lemma="strong:G1161 lemma.TR:δε" morph="robinson:CONJ">And</w> <w src="8" lemma="strong:G2962 lemma.TR:κυριος" morph="robinson:N-NSM">the Lord</w> <w src="9" lemma="strong:G3004 lemma.TR:ειπεν" morph="robinson:V-2AAI-3S">said</w>, <q who="Jesus" marker=""><w src="10" lemma="strong:G1473 lemma.TR:εγω" morph="robinson:P-1NS">I</w> <w src="11" lemma="strong:G1510 lemma.TR:ειμι" morph="robinson:V-PAI-1S">am</w> <w src="12" lemma="strong:G2424 lemma.TR:ιησους" morph="robinson:N-NSM">Jesus</w> <w src="13" lemma="strong:G3739 lemma.TR:ον" morph="robinson:R-ASM">whom</w> <w src="14" lemma="strong:G4771 lemma.TR:συ" morph="robinson:P-2NS">thou</w> <w src="15" lemma="strong:G1377 lemma.TR:διωκεις" morph="robinson:V-PAI-2S">persecutest</w>: <transChange type="added">it is</transChange> <w src="16" lemma="strong:G4642 lemma.TR:σκληρον" morph="robinson:A-NSN">hard</w> <w src="17" lemma="strong:G4771 lemma.TR:σοι" morph="robinson:P-2DS">for thee</w> <w src="20" lemma="strong:G2979 lemma.TR:λακτιζειν" morph="robinson:V-PAN">to kick</w> <w src="18" lemma="strong:G4314 lemma.TR:προς" morph="robinson:PREP">against</w> <w src="19" lemma="strong:G2759 lemma.TR:κεντρα" morph="robinson:N-APN">the pricks</w>.</q><verse eID="Acts.9.5"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Acts.9.5" sID="Acts.9.5"/><w/><w>And</w> <w>he said</w>, <w>Who</w> <w>art thou</w>, <w>Lord</w>? <w>And</w> <w>the Lord</w> <w>said</w>, <q who="Jesus" marker=""><w>I</w> <w>am</w> <w>Jesus</w> <w>whom</w> <w>thou</w> <w>persecutest</w>: <transChange type="added">it is</transChange> <w>hard</w> <w>for thee</w> <w>to kick</w> <w>against</w> <w>the pricks</w>.</q><verse eID="Acts.9.5"/>
```

### ACT.9.6 (OSIS `Acts.9.6`)

- offset 21735575, length 2588, excerpt SHA-256 `255458a11ed2296c2128b6fae54adbc39f5028299188b120ed71f1a4db5670eb`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 1; `</q>` end tags in excerpt: 1

```xml
<verse osisID="Acts.9.6" sID="Acts.9.6"/><w src="2" lemma="strong:G5037 lemma.TR:τε" morph="robinson:PRT">And</w> <w src="1" lemma="strong:G5141 lemma.TR:τρεμων" morph="robinson:V-PAP-NSM">he trembling</w> <w src="3" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="4" lemma="strong:G2284 lemma.TR:θαμβων" morph="robinson:V-PAP-NSM">astonished</w> <w src="5" lemma="strong:G3004 lemma.TR:ειπεν" morph="robinson:V-2AAI-3S">said</w>, <w src="6" lemma="strong:G2962 lemma.TR:κυριε" morph="robinson:N-VSM">Lord</w>, <w src="7" lemma="strong:G5101 lemma.TR:τι" morph="robinson:I-ASN">what</w> <w src="9" lemma="strong:G2309 lemma.TR:θελεις" morph="robinson:V-PAI-2S">wilt thou</w> <w src="10" lemma="strong:G4160 lemma.TR:ποιησαι" morph="robinson:V-AAN" type="x-split-2275">have</w> <w src="8" lemma="strong:G1473 lemma.TR:με" morph="robinson:P-1AS">me</w> <w src="10" lemma="strong:G4160 lemma.TR:ποιησαι" morph="robinson:V-AAN" type="x-split-2275">to do</w>? <w src="11" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">And</w> <w src="12 13" lemma="strong:G3588 strong:G2962 lemma.TR:ο lemma.TR:κυριος" morph="robinson:T-NSM robinson:N-NSM">the Lord</w> <transChange type="added">said</transChange> <w src="14" lemma="strong:G4314 lemma.TR:προς" morph="robinson:PREP">unto</w> <w src="15" lemma="strong:G846 lemma.TR:αυτον" morph="robinson:P-ASM">him</w>, <q who="Jesus" marker=""><w src="16" lemma="strong:G450 lemma.TR:αναστηθι" morph="robinson:V-2AAM-2S">Arise</w>, <w src="17" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="18" lemma="strong:G1525 lemma.TR:εισελθε" morph="robinson:V-2AAM-2S">go</w> <w src="19" lemma="strong:G1519 lemma.TR:εις" morph="robinson:PREP">into</w> <w src="20 21" lemma="strong:G3588 strong:G4172 lemma.TR:την lemma.TR:πολιν" morph="robinson:T-ASF robinson:N-ASF">the city</w>, <w src="22" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="23" lemma="strong:G2980 lemma.TR:λαληθησεται" morph="robinson:V-FPI-3S">it shall be told</w> <w src="24" lemma="strong:G4771 lemma.TR:σοι" morph="robinson:P-2DS">thee</w> <w src="25" lemma="strong:G5101 lemma.TR:τι" morph="robinson:I-ASN">what</w> <w src="26" lemma="strong:G4771 lemma.TR:σε" morph="robinson:P-2AS">thou</w> <w src="27" lemma="strong:G1163 lemma.TR:δει" morph="robinson:V-PAI-3S">must</w> <w src="28" lemma="strong:G4160 lemma.TR:ποιειν" morph="robinson:V-PAN">do</w>.</q><verse eID="Acts.9.6"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Acts.9.6" sID="Acts.9.6"/><w>And</w> <w>he trembling</w> <w>and</w> <w>astonished</w> <w>said</w>, <w>Lord</w>, <w>what</w> <w>wilt thou</w> <w>have</w> <w>me</w> <w>to do</w>? <w>And</w> <w>the Lord</w> <transChange type="added">said</transChange> <w>unto</w> <w>him</w>, <q who="Jesus" marker=""><w>Arise</w>, <w>and</w> <w>go</w> <w>into</w> <w>the city</w>, <w>and</w> <w>it shall be told</w> <w>thee</w> <w>what</w> <w>thou</w> <w>must</w> <w>do</w>.</q><verse eID="Acts.9.6"/>
```

### ACT.9.10 (OSIS `Acts.9.10`)

- offset 21742407, length 2083, excerpt SHA-256 `21d43c82d6c8465a03b318d213b2e106b3e10b8473a4a9588dda632d414ea768`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 1; `</q>` end tags in excerpt: 1

```xml
<verse osisID="Acts.9.10" sID="Acts.9.10"/><milestone type="x-p" marker="¶"/><w src="18" lemma="strong:G3588 lemma.TR:ο" morph="robinson:T-NSM"/><w src="2" lemma="strong:G1161 lemma.TR:δε" morph="robinson:CONJ">And</w> <w src="1" lemma="strong:G1510 lemma.TR:ην" morph="robinson:V-IAI-3S">there was</w> <w src="3" lemma="strong:G5100 lemma.TR:τις" morph="robinson:X-NSM">a certain</w> <w src="4" lemma="strong:G3101 lemma.TR:μαθητης" morph="robinson:N-NSM">disciple</w> <w src="5" lemma="strong:G1722 lemma.TR:εν" morph="robinson:PREP">at</w> <w src="6" lemma="strong:G1154 lemma.TR:δαμασκω" morph="robinson:N-DSF">Damascus</w>, <w src="7" lemma="strong:G3686 lemma.TR:ονοματι" morph="robinson:N-DSN">named</w> <w src="8" lemma="strong:G367 lemma.TR:ανανιας" morph="robinson:N-NSM">Ananias</w>; <w src="9" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="11" lemma="strong:G4314 lemma.TR:προς" morph="robinson:PREP">to</w> <w src="12" lemma="strong:G846 lemma.TR:αυτον" morph="robinson:P-ASM">him</w> <w src="10" lemma="strong:G3004 lemma.TR:ειπεν" morph="robinson:V-2AAI-3S">said</w> <w src="13 14" lemma="strong:G3588 strong:G2962 lemma.TR:ο lemma.TR:κυριος" morph="robinson:T-NSM robinson:N-NSM">the Lord</w> <w src="15" lemma="strong:G1722 lemma.TR:εν" morph="robinson:PREP">in</w> <w src="16" lemma="strong:G3705 lemma.TR:οραματι" morph="robinson:N-DSN">a vision</w>, <q who="Jesus" marker=""><w src="17" lemma="strong:G367 lemma.TR:ανανια" morph="robinson:N-VSM">Ananias</w>.</q> <w src="19" lemma="strong:G1161 lemma.TR:δε" morph="robinson:CONJ">And</w> <w src="20" lemma="strong:G3004 lemma.TR:ειπεν" morph="robinson:V-2AAI-3S">he said</w>, <w src="21" lemma="strong:G3708 lemma.TR:ιδου" morph="robinson:V-2AMM-2S">Behold</w>, <w src="22" lemma="strong:G1473 lemma.TR:εγω" morph="robinson:P-1NS">I</w> <transChange type="added">am here</transChange>, <w src="23" lemma="strong:G2962 lemma.TR:κυριε" morph="robinson:N-VSM">Lord</w>.<verse eID="Acts.9.10"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Acts.9.10" sID="Acts.9.10"/><milestone type="x-p" marker="¶"/><w/><w>And</w> <w>there was</w> <w>a certain</w> <w>disciple</w> <w>at</w> <w>Damascus</w>, <w>named</w> <w>Ananias</w>; <w>and</w> <w>to</w> <w>him</w> <w>said</w> <w>the Lord</w> <w>in</w> <w>a vision</w>, <q who="Jesus" marker=""><w>Ananias</w>.</q> <w>And</w> <w>he said</w>, <w>Behold</w>, <w>I</w> <transChange type="added">am here</transChange>, <w>Lord</w>.<verse eID="Acts.9.10"/>
```

### ACT.20.35 (OSIS `Acts.20.35`)

- offset 22468863, length 2340, excerpt SHA-256 `919344c6580331dcafeefe08cbcee472c4460a566a3f2650299cdc70afe665ca`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 1; `</q>` end tags in excerpt: 1

```xml
<verse osisID="Acts.20.35" sID="Acts.20.35"/><w src="9" lemma="strong:G3588 lemma.TR:των" morph="robinson:T-GPM"/><w src="2" lemma="strong:G5263 lemma.TR:υπεδειξα" morph="robinson:V-AAI-1S">I have shewed</w> <w src="3" lemma="strong:G4771 lemma.TR:υμιν" morph="robinson:P-2DP">you</w> <w src="1" lemma="strong:G3956 lemma.TR:παντα" morph="robinson:A-APN">all things</w>, <w src="4" lemma="strong:G3754 lemma.TR:οτι" morph="robinson:CONJ">how that</w> <w src="5" lemma="strong:G3779 lemma.TR:ουτως" morph="robinson:ADV">so</w> <w src="6" lemma="strong:G2872 lemma.TR:κοπιωντας" morph="robinson:V-PAP-APM">labouring</w> <w src="7" lemma="strong:G1163 lemma.TR:δει" morph="robinson:V-PAI-3S">ye ought</w> <w src="8" lemma="strong:G482 lemma.TR:αντιλαμβανεσθαι" morph="robinson:V-PNN">to support</w> <w src="10" lemma="strong:G770 lemma.TR:ασθενουντων" morph="robinson:V-PAP-GPM">the weak</w>, <w src="12" lemma="strong:G5037 lemma.TR:τε" morph="robinson:PRT">and</w> <w src="11" lemma="strong:G3421 lemma.TR:μνημονευειν" morph="robinson:V-PAN">to remember</w> <w src="13 14" lemma="strong:G3588 strong:G3056 lemma.TR:των lemma.TR:λογων" morph="robinson:T-GPM robinson:N-GPM">the words</w> <w src="15 16" lemma="strong:G3588 strong:G2962 lemma.TR:του lemma.TR:κυριου" morph="robinson:T-GSM robinson:N-GSM">of the Lord</w> <w src="17" lemma="strong:G2424 lemma.TR:ιησου" morph="robinson:N-GSM">Jesus</w>, <w src="18" lemma="strong:G3754 lemma.TR:οτι" morph="robinson:CONJ">how</w> <w src="19" lemma="strong:G846 lemma.TR:αυτος" morph="robinson:P-NSM">he</w> <w src="20" lemma="strong:G3004 lemma.TR:ειπεν" morph="robinson:V-2AAI-3S">said</w>, <q who="Jesus" marker=""><w src="22" lemma="strong:G1510 lemma.TR:εστιν" morph="robinson:V-PAI-3S">It is</w> <w src="24" lemma="strong:G3123 lemma.TR:μαλλον" morph="robinson:ADV">more</w> <w src="21" lemma="strong:G3107 lemma.TR:μακαριον" morph="robinson:A-NSN">blessed</w> <w src="23" lemma="strong:G1325 lemma.TR:διδοναι" morph="robinson:V-PAN">to give</w> <w src="25" lemma="strong:G2228 lemma.TR:η" morph="robinson:PRT">than</w> <w src="26" lemma="strong:G2983 lemma.TR:λαμβανειν" morph="robinson:V-PAN">to receive</w>.</q><verse eID="Acts.20.35"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Acts.20.35" sID="Acts.20.35"/><w/><w>I have shewed</w> <w>you</w> <w>all things</w>, <w>how that</w> <w>so</w> <w>labouring</w> <w>ye ought</w> <w>to support</w> <w>the weak</w>, <w>and</w> <w>to remember</w> <w>the words</w> <w>of the Lord</w> <w>Jesus</w>, <w>how</w> <w>he</w> <w>said</w>, <q who="Jesus" marker=""><w>It is</w> <w>more</w> <w>blessed</w> <w>to give</w> <w>than</w> <w>to receive</w>.</q><verse eID="Acts.20.35"/>
```

### 1CO.11.24 (OSIS `1Cor.11.24`)

- offset 23998924, length 2016, excerpt SHA-256 `12493bdad40633e3703fffa3755921db9b06663f0da78fa3685f8cadb74de44c`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 1; `</q>` end tags in excerpt: 1

```xml
<verse osisID="1Cor.11.24" sID="1Cor.11.24"/><w src="20" lemma="strong:G3588 lemma.TR:την" morph="robinson:T-ASF"/><w src="13" lemma="strong:G3588 lemma.TR:το" morph="robinson:T-NSN"/><w src="1" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">And</w> <w src="2" lemma="strong:G2168 lemma.TR:ευχαριστησας" morph="robinson:V-AAP-NSM">when he had given thanks</w>, <w src="3" lemma="strong:G2806 lemma.TR:εκλασεν" morph="robinson:V-AAI-3S">he brake</w> <transChange type="added">it</transChange>, <w src="4" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="5" lemma="strong:G3004 lemma.TR:ειπεν" morph="robinson:V-2AAI-3S">said</w>, <q who="Jesus" marker=""><w src="6" lemma="strong:G2983 lemma.TR:λαβετε" morph="robinson:V-2AAM-2P">Take</w>, <w src="7" lemma="strong:G5315 lemma.TR:φαγετε" morph="robinson:V-2AAM-2P">eat</w>: <w src="8" lemma="strong:G3778 lemma.TR:τουτο" morph="robinson:D-NSN">this</w> <w src="10" lemma="strong:G1510 lemma.TR:εστιν" morph="robinson:V-PAI-3S">is</w> <w src="9" lemma="strong:G1473 lemma.TR:μου" morph="robinson:P-1GS">my</w> <w src="11 12" lemma="strong:G3588 strong:G4983 lemma.TR:το lemma.TR:σωμα" morph="robinson:T-NSN robinson:N-NSN">body</w>, <w src="16" lemma="strong:G2806 lemma.TR:κλωμενον" morph="robinson:V-PPP-NSN">which is broken</w> <w src="14" lemma="strong:G5228 lemma.TR:υπερ" morph="robinson:PREP">for</w> <w src="15" lemma="strong:G4771 lemma.TR:υμων" morph="robinson:P-2GP">you</w>: <w src="17" lemma="strong:G3778 lemma.TR:τουτο" morph="robinson:D-ASN">this</w> <w src="18" lemma="strong:G4160 lemma.TR:ποιειτε" morph="robinson:V-PAM-2P">do</w> <w src="19" lemma="strong:G1519 lemma.TR:εις" morph="robinson:PREP">in</w> <w src="22" lemma="strong:G364 lemma.TR:αναμνησιν" morph="robinson:N-ASF">remembrance</w> <w src="21" lemma="strong:G1699 lemma.TR:εμην" morph="robinson:S-1SASF">of me</w>.</q><verse eID="1Cor.11.24"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="1Cor.11.24" sID="1Cor.11.24"/><w/><w/><w>And</w> <w>when he had given thanks</w>, <w>he brake</w> <transChange type="added">it</transChange>, <w>and</w> <w>said</w>, <q who="Jesus" marker=""><w>Take</w>, <w>eat</w>: <w>this</w> <w>is</w> <w>my</w> <w>body</w>, <w>which is broken</w> <w>for</w> <w>you</w>: <w>this</w> <w>do</w> <w>in</w> <w>remembrance</w> <w>of me</w>.</q><verse eID="1Cor.11.24"/>
```

### 1CO.11.25 (OSIS `1Cor.11.25`)

- offset 24000941, length 2416, excerpt SHA-256 `f2b9624f1d707515b88eb96487a12d8c35651088764b2fc69b08445eaa05e7cd`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 1; `</q>` end tags in excerpt: 1

```xml
<verse osisID="1Cor.11.25" sID="1Cor.11.25"/><w src="1" lemma="strong:G5615 lemma.TR:ωσαυτως" morph="robinson:ADV">After the same manner</w> <w src="2" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">also</w> <transChange type="added">he took</transChange> <w src="3 4" lemma="strong:G3588 strong:G4221 lemma.TR:το lemma.TR:ποτηριον" morph="robinson:T-NSN robinson:N-NSN">the cup</w>, <w src="5" lemma="strong:G3326 lemma.TR:μετα" morph="robinson:PREP">when</w> <w src="6" lemma="strong:G3588 lemma.TR:το" morph="robinson:T-ASN"/><w src="7" lemma="strong:G1172 lemma.TR:δειπνησαι" morph="robinson:V-AAN">he had supped</w>, <w src="8" lemma="strong:G3004 lemma.TR:λεγων" morph="robinson:V-PAP-NSM">saying</w>, <q who="Jesus" marker=""><w src="9" lemma="strong:G3778 lemma.TR:τουτο" morph="robinson:D-NSN">This</w> <w src="10 11" lemma="strong:G3588 strong:G4221 lemma.TR:το lemma.TR:ποτηριον" morph="robinson:T-NSN robinson:N-NSN">cup</w> <w src="15" lemma="strong:G1510 lemma.TR:εστιν" morph="robinson:V-PAI-3S">is</w> <w src="12 13" lemma="strong:G3588 strong:G2537 lemma.TR:η lemma.TR:καινη" morph="robinson:T-NSF robinson:A-NSF">the new</w> <w src="14" lemma="strong:G1242 lemma.TR:διαθηκη" morph="robinson:N-NSF">testament</w> <w src="16" lemma="strong:G1722 lemma.TR:εν" morph="robinson:PREP">in</w> <w src="17 18" lemma="strong:G3588 strong:G1699 lemma.TR:τω lemma.TR:εμω" morph="robinson:T-DSN robinson:S-1SDSN">my</w> <w src="19" lemma="strong:G129 lemma.TR:αιματι" morph="robinson:N-DSN">blood</w>: <w src="20" lemma="strong:G3778 lemma.TR:τουτο" morph="robinson:D-ASN">this</w> <w src="21" lemma="strong:G4160 lemma.TR:ποιειτε" morph="robinson:V-PAM-2P">do ye</w>, <w src="22" lemma="strong:G3740 lemma.TR:οσακις" morph="robinson:ADV">as oft</w> <w src="24" lemma="strong:G4095 lemma.TR:πινητε" morph="robinson:V-PAS-2P">as ye drink</w> <w src="23" lemma="strong:G302 lemma.TR:αν" morph="robinson:PRT"/><transChange type="added">it</transChange>, <w src="25" lemma="strong:G1519 lemma.TR:εις" morph="robinson:PREP">in</w> <w src="28" lemma="strong:G364 lemma.TR:αναμνησιν" morph="robinson:N-ASF">remembrance</w> <w src="26 27" lemma="strong:G3588 strong:G1699 lemma.TR:την lemma.TR:εμην" morph="robinson:T-ASF robinson:S-1SASF">of me</w>.</q><verse eID="1Cor.11.25"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="1Cor.11.25" sID="1Cor.11.25"/><w>After the same manner</w> <w>also</w> <transChange type="added">he took</transChange> <w>the cup</w>, <w>when</w> <w/><w>he had supped</w>, <w>saying</w>, <q who="Jesus" marker=""><w>This</w> <w>cup</w> <w>is</w> <w>the new</w> <w>testament</w> <w>in</w> <w>my</w> <w>blood</w>: <w>this</w> <w>do ye</w>, <w>as oft</w> <w>as ye drink</w> <w/><transChange type="added">it</transChange>, <w>in</w> <w>remembrance</w> <w>of me</w>.</q><verse eID="1Cor.11.25"/>
```

### 1CO.11.26 (OSIS `1Cor.11.26`)

- offset 24003358, length 1703, excerpt SHA-256 `d5e91b4ee064d03ec7c80538e862f58adfe194d0b0b26e187094f6f1a9673634`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="1Cor.11.26" sID="1Cor.11.26"/><w src="2" lemma="strong:G1063 lemma.TR:γαρ" morph="robinson:CONJ">For</w> <w src="1" lemma="strong:G3740 lemma.TR:οσακις" morph="robinson:ADV">as often as</w> <w src="4" lemma="strong:G2068 lemma.TR:εσθιητε" morph="robinson:V-PAS-2P">ye eat</w> <w src="3" lemma="strong:G302 lemma.TR:αν" morph="robinson:PRT"/><w src="7" lemma="strong:G3778 lemma.TR:τουτον" morph="robinson:D-ASM">this</w> <w src="5 6" lemma="strong:G3588 strong:G740 lemma.TR:τον lemma.TR:αρτον" morph="robinson:T-ASM robinson:N-ASM">bread</w>, <w src="8" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="12" lemma="strong:G4095 lemma.TR:πινητε" morph="robinson:V-PAS-2P">drink</w> <w src="11" lemma="strong:G3778 lemma.TR:τουτο" morph="robinson:D-ASN">this</w> <w src="9 10" lemma="strong:G3588 strong:G4221 lemma.TR:το lemma.TR:ποτηριον" morph="robinson:T-ASN robinson:N-ASN">cup</w>, <w src="17" lemma="strong:G2605 lemma.TR:καταγγελλετε" morph="robinson:V-PAI-2P">ye do shew</w> <w src="15 16" lemma="strong:G3588 strong:G2962 lemma.TR:του lemma.TR:κυριου" morph="robinson:T-GSM robinson:N-GSM">the Lord’s</w> <w src="13 14" lemma="strong:G3588 strong:G2288 lemma.TR:τον lemma.TR:θανατον" morph="robinson:T-ASM robinson:N-ASM">death</w> <w src="18" lemma="strong:G891 lemma.TR:αχρις" morph="robinson:ADV">till</w> <w src="19" lemma="strong:G3739 lemma.TR:ου" morph="robinson:R-GSM"/><w src="20" lemma="strong:G302 lemma.TR:αν" morph="robinson:PRT"/><w src="21" lemma="strong:G2064 lemma.TR:ελθη" morph="robinson:V-2AAS-3S">he come</w>.<verse eID="1Cor.11.26"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="1Cor.11.26" sID="1Cor.11.26"/><w>For</w> <w>as often as</w> <w>ye eat</w> <w/><w>this</w> <w>bread</w>, <w>and</w> <w>drink</w> <w>this</w> <w>cup</w>, <w>ye do shew</w> <w>the Lord’s</w> <w>death</w> <w>till</w> <w/><w/><w>he come</w>.<verse eID="1Cor.11.26"/>
```

### 2CO.12.9 (OSIS `2Cor.12.9`)

- offset 24618895, length 2768, excerpt SHA-256 `ff89dac93e36ff69e6badd252bad51981bc60c740bd2120f23205d6d3d9e4c2f`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 1; `</q>` end tags in excerpt: 1

```xml
<verse osisID="2Cor.12.9" sID="2Cor.12.9"/><w src="9" lemma="strong:G3588 lemma.TR:η" morph="robinson:T-NSF"/><w src="1" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">And</w> <w src="2" lemma="strong:G2046 lemma.TR:ειρηκεν" morph="robinson:V-RAI-3S-ATT">he said</w> <w src="3" lemma="strong:G1473 lemma.TR:μοι" morph="robinson:P-1DS">unto me</w>, <q who="Jesus" marker=""><w src="8" lemma="strong:G1473 lemma.TR:μου" morph="robinson:P-1GS">My</w> <w src="6 7" lemma="strong:G3588 strong:G5485 lemma.TR:η lemma.TR:χαρις" morph="robinson:T-NSF robinson:N-NSF">grace</w> <w src="4" lemma="strong:G714 lemma.TR:αρκει" morph="robinson:V-PAI-3S">is sufficient</w> <w src="5" lemma="strong:G4771 lemma.TR:σοι" morph="robinson:P-2DS">for thee</w>: <w src="10" lemma="strong:G1063 lemma.TR:γαρ" morph="robinson:CONJ">for</w> <w src="12" lemma="strong:G1473 lemma.TR:μου" morph="robinson:P-1GS">my</w> <w src="11" lemma="strong:G1411 lemma.TR:δυναμις" morph="robinson:N-NSF">strength</w> <w src="15" lemma="strong:G5048 lemma.TR:τελειουται" morph="robinson:V-PPI-3S">is made perfect</w> <w src="13" lemma="strong:G1722 lemma.TR:εν" morph="robinson:PREP">in</w> <w src="14" lemma="strong:G769 lemma.TR:ασθενεια" morph="robinson:N-DSF">weakness</w>.</q> <w src="16" lemma="strong:G2236 lemma.TR:ηδιστα" morph="robinson:ADV-S">Most gladly</w> <w src="17" lemma="strong:G3767 lemma.TR:ουν" morph="robinson:CONJ">therefore</w> <w src="19" lemma="strong:G2744 lemma.TR:καυχησομαι" morph="robinson:V-FDI-1S" type="x-split-2947">will I</w> <w src="18" lemma="strong:G3123 lemma.TR:μαλλον" morph="robinson:ADV">rather</w> <w src="19" lemma="strong:G2744 lemma.TR:καυχησομαι" morph="robinson:V-FDI-1S" type="x-split-2947">glory</w> <w src="20" lemma="strong:G1722 lemma.TR:εν" morph="robinson:PREP">in</w> <w src="23" lemma="strong:G1473 lemma.TR:μου" morph="robinson:P-1GS">my</w> <w src="21 22" lemma="strong:G3588 strong:G769 lemma.TR:ταις lemma.TR:ασθενειαις" morph="robinson:T-DPF robinson:N-DPF">infirmities</w>, <w src="24" lemma="strong:G2443 lemma.TR:ινα" morph="robinson:CONJ">that</w> <w src="28 29" lemma="strong:G3588 strong:G1411 lemma.TR:η lemma.TR:δυναμις" morph="robinson:T-NSF robinson:N-NSF">the power</w> <w src="30 31" lemma="strong:G3588 strong:G5547 lemma.TR:του lemma.TR:χριστου" morph="robinson:T-GSM robinson:N-GSM">of Christ</w> <w src="25" lemma="strong:G1981 lemma.TR:επισκηνωση" morph="robinson:V-AAS-3S">may rest</w> <w src="26" lemma="strong:G1909 lemma.TR:επ" morph="robinson:PREP">upon</w> <w src="27" lemma="strong:G1473 lemma.TR:εμε" morph="robinson:P-1AS">me</w>.<verse eID="2Cor.12.9"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="2Cor.12.9" sID="2Cor.12.9"/><w/><w>And</w> <w>he said</w> <w>unto me</w>, <q who="Jesus" marker=""><w>My</w> <w>grace</w> <w>is sufficient</w> <w>for thee</w>: <w>for</w> <w>my</w> <w>strength</w> <w>is made perfect</w> <w>in</w> <w>weakness</w>.</q> <w>Most gladly</w> <w>therefore</w> <w>will I</w> <w>rather</w> <w>glory</w> <w>in</w> <w>my</w> <w>infirmities</w>, <w>that</w> <w>the power</w> <w>of Christ</w> <w>may rest</w> <w>upon</w> <w>me</w>.<verse eID="2Cor.12.9"/>
```

### REV.1.7 (OSIS `Rev.1.7`)

- offset 27195837, length 2284, excerpt SHA-256 `0adc362629d77fd91424ff880900f2e1ff98fd4261c1a7d4c3422398dd4eaac9`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="Rev.1.7" sID="Rev.1.7"/><w src="1" lemma="strong:G3708 lemma.TR:ιδου" morph="robinson:V-2AMM-2S">Behold</w>, <w src="2" lemma="strong:G2064 lemma.TR:ερχεται" morph="robinson:V-PNI-3S">he cometh</w> <w src="3" lemma="strong:G3326 lemma.TR:μετα" morph="robinson:PREP">with</w> <w src="4 5" lemma="strong:G3588 strong:G3507 lemma.TR:των lemma.TR:νεφελων" morph="robinson:T-GPF robinson:N-GPF">clouds</w>; <w src="6" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="9" lemma="strong:G3956 lemma.TR:πας" morph="robinson:A-NSM">every</w> <w src="10" lemma="strong:G3788 lemma.TR:οφθαλμος" morph="robinson:N-NSM">eye</w> <w src="7" lemma="strong:G3708 lemma.TR:οψεται" morph="robinson:V-FDI-3S">shall see</w> <w src="8" lemma="strong:G846 lemma.TR:αυτον" morph="robinson:P-ASM">him</w>, <w src="11" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="12" lemma="strong:G3748 lemma.TR:οιτινες" morph="robinson:R-NPM" type="x-split-3732">they</w> <transChange type="added">also</transChange> <w src="12" lemma="strong:G3748 lemma.TR:οιτινες" morph="robinson:R-NPM" type="x-split-3732">which</w> <w src="14" lemma="strong:G1574 lemma.TR:εξεκεντησαν" morph="robinson:V-AAI-3P">pierced</w> <w src="13" lemma="strong:G846 lemma.TR:αυτον" morph="robinson:P-ASM">him</w>: <w src="15" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="19" lemma="strong:G3956 lemma.TR:πασαι" morph="robinson:A-NPF">all</w> <w src="20 21" lemma="strong:G3588 strong:G5443 lemma.TR:αι lemma.TR:φυλαι" morph="robinson:T-NPF robinson:N-NPF">kindreds</w> <w src="22 23" lemma="strong:G3588 strong:G1093 lemma.TR:της lemma.TR:γης" morph="robinson:T-GSF robinson:N-GSF">of the earth</w> <w src="16" lemma="strong:G2875 lemma.TR:κοψονται" morph="robinson:V-FDI-3P">shall wail</w> <w src="17" lemma="strong:G1909 lemma.TR:επ" morph="robinson:PREP">because of</w> <w src="18" lemma="strong:G846 lemma.TR:αυτον" morph="robinson:P-ASM">him</w>. <w src="24" lemma="strong:G3483 lemma.TR:ναι" morph="robinson:PRT">Even so</w>, <w src="25" lemma="strong:G281 lemma.TR:αμην" morph="robinson:HEB">Amen</w>.<verse eID="Rev.1.7"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Rev.1.7" sID="Rev.1.7"/><w>Behold</w>, <w>he cometh</w> <w>with</w> <w>clouds</w>; <w>and</w> <w>every</w> <w>eye</w> <w>shall see</w> <w>him</w>, <w>and</w> <w>they</w> <transChange type="added">also</transChange> <w>which</w> <w>pierced</w> <w>him</w>: <w>and</w> <w>all</w> <w>kindreds</w> <w>of the earth</w> <w>shall wail</w> <w>because of</w> <w>him</w>. <w>Even so</w>, <w>Amen</w>.<verse eID="Rev.1.7"/>
```

### REV.1.8 (OSIS `Rev.1.8`)

- offset 27198122, length 1742, excerpt SHA-256 `61de98bab731b7dd88c07952b425a1a7615faabe53aebaa99c0cf2b8e30e0380`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="Rev.1.8" sID="Rev.1.8"/><w src="1" lemma="strong:G1473 lemma.TR:εγω" morph="robinson:P-1NS">I</w> <w src="2" lemma="strong:G1510 lemma.TR:ειμι" morph="robinson:V-PAI-1S">am</w> <w src="3 4" lemma="strong:G3588 strong:G1 lemma.TR:το lemma.TR:α" morph="robinson:T-NSN robinson:N-LI">Alpha</w> <w src="5" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="6 7" lemma="strong:G3588 strong:G5598 lemma.TR:το lemma.TR:ω" morph="robinson:T-NSN robinson:N-LI">Omega</w>, <w src="8" lemma="strong:G746 lemma.TR:αρχη" morph="robinson:N-NSF">the beginning</w> <w src="9" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="10" lemma="strong:G5056 lemma.TR:τελος" morph="robinson:N-NSN">the ending</w>, <w src="11" lemma="strong:G3004 lemma.TR:λεγει" morph="robinson:V-PAI-3S">saith</w> <w src="12 13" lemma="strong:G3588 strong:G2962 lemma.TR:ο lemma.TR:κυριος" morph="robinson:T-NSM robinson:N-NSM">the Lord</w>, <w src="14 15" lemma="strong:G3588 strong:G1510 lemma.TR:ο lemma.TR:ων" morph="robinson:T-NSM robinson:V-PAP-NSM">which is</w>, <w src="16" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="17 18" lemma="strong:G3588 strong:G1510 lemma.TR:ο lemma.TR:ην" morph="robinson:T-NSM robinson:V-IAI-3S">which was</w>, <w src="19" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="20 21" lemma="strong:G3588 strong:G2064 lemma.TR:ο lemma.TR:ερχομενος" morph="robinson:T-NSM robinson:V-PNP-NSM">which is to come</w>, <w src="22 23" lemma="strong:G3588 strong:G3841 lemma.TR:ο lemma.TR:παντοκρατωρ" morph="robinson:T-NSM robinson:N-NSM">the Almighty</w>.<verse eID="Rev.1.8"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Rev.1.8" sID="Rev.1.8"/><w>I</w> <w>am</w> <w>Alpha</w> <w>and</w> <w>Omega</w>, <w>the beginning</w> <w>and</w> <w>the ending</w>, <w>saith</w> <w>the Lord</w>, <w>which is</w>, <w>and</w> <w>which was</w>, <w>and</w> <w>which is to come</w>, <w>the Almighty</w>.<verse eID="Rev.1.8"/>
```

### REV.1.11 (OSIS `Rev.1.11`)

- offset 27204447, length 4288, excerpt SHA-256 `b7296e0df727103222e936060b970d43b5339b2bfd0de001acc1a412a56e498e`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 2; `</q>` end tags in excerpt: 2

```xml
<verse osisID="Rev.1.11" sID="Rev.1.11"/><w src="1" lemma="strong:G3004 lemma.TR:λεγουσης" morph="robinson:V-PAP-GSF">Saying</w>, <q who="Jesus" marker=""><w src="2" lemma="strong:G1473 lemma.TR:εγω" morph="robinson:P-1NS">I</w> <w src="3" lemma="strong:G1510 lemma.TR:ειμι" morph="robinson:V-PAI-1S">am</w> <w src="4 5" lemma="strong:G3588 strong:G1 lemma.TR:το lemma.TR:α" morph="robinson:T-NSN robinson:N-LI">Alpha</w> <w src="6" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="7 8" lemma="strong:G3588 strong:G5598 lemma.TR:το lemma.TR:ω" morph="robinson:T-NSN robinson:N-LI">Omega</w>, <w src="9 10" lemma="strong:G3588 strong:G4413 lemma.TR:ο lemma.TR:πρωτος" morph="robinson:T-NSM robinson:A-NSM-S">the first</w> <w src="11" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="12 13" lemma="strong:G3588 strong:G2078 lemma.TR:ο lemma.TR:εσχατος" morph="robinson:T-NSM robinson:A-NSM-S">the last</w>:</q> <w src="14" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w>, <q who="Jesus" marker=""><w src="15" lemma="strong:G3739 lemma.TR:ο" morph="robinson:R-ASN">What</w> <w src="16" lemma="strong:G991 lemma.TR:βλεπεις" morph="robinson:V-PAI-2S">thou seest</w>, <w src="17" lemma="strong:G1125 lemma.TR:γραψον" morph="robinson:V-AAM-2S">write</w> <w src="18" lemma="strong:G1519 lemma.TR:εις" morph="robinson:PREP">in</w> <w src="19" lemma="strong:G975 lemma.TR:βιβλιον" morph="robinson:N-ASN">a book</w>, <w src="20" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="21" lemma="strong:G3992 lemma.TR:πεμψον" morph="robinson:V-AAM-2S">send</w> <transChange type="added">it</transChange> <w src="22 24" lemma="strong:G3588 strong:G1577 lemma.TR:ταις lemma.TR:εκκλησιαις" morph="robinson:T-DPF robinson:N-DPF" type="x-split-3735">unto the</w> <w src="23" lemma="strong:G2033 lemma.TR:επτα" morph="robinson:A-NUI">seven</w> <w src="22 24" lemma="strong:G3588 strong:G1577 lemma.TR:ταις lemma.TR:εκκλησιαις" morph="robinson:T-DPF robinson:N-DPF" type="x-split-3735">churches</w> <w src="25 27" lemma="strong:G3588 strong:G773 lemma.TR:ταις lemma.TR:ασια" morph="robinson:T-DPF robinson:N-DSF" type="x-split-3736">which are</w> <w src="26" lemma="strong:G1722 lemma.TR:εν" morph="robinson:PREP">in</w> <w src="25 27" lemma="strong:G3588 strong:G773 lemma.TR:ταις lemma.TR:ασια" morph="robinson:T-DPF robinson:N-DSF" type="x-split-3736">Asia</w>; <w src="28" lemma="strong:G1519 lemma.TR:εις" morph="robinson:PREP">unto</w> <w src="29" lemma="strong:G2181 lemma.TR:εφεσον" morph="robinson:N-ASF">Ephesus</w>, <w src="30" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="31" lemma="strong:G1519 lemma.TR:εις" morph="robinson:PREP">unto</w> <w src="32" lemma="strong:G4667 lemma.TR:σμυρναν" morph="robinson:N-ASF">Smyrna</w>, <w src="33" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="34" lemma="strong:G1519 lemma.TR:εις" morph="robinson:PREP">unto</w> <w src="35" lemma="strong:G4010 lemma.TR:περγαμον" morph="robinson:N-ASF">Pergamos</w>, <w src="36" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="37" lemma="strong:G1519 lemma.TR:εις" morph="robinson:PREP">unto</w> <w src="38" lemma="strong:G2363 lemma.TR:θυατειρα" morph="robinson:N-APN">Thyatira</w>, <w src="39" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="40" lemma="strong:G1519 lemma.TR:εις" morph="robinson:PREP">unto</w> <w src="41" lemma="strong:G4554 lemma.TR:σαρδεις" morph="robinson:N-APF">Sardis</w>, <w src="42" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="43" lemma="strong:G1519 lemma.TR:εις" morph="robinson:PREP">unto</w> <w src="44" lemma="strong:G5359 lemma.TR:φιλαδελφειαν" morph="robinson:N-ASF">Philadelphia</w>, <w src="45" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="46" lemma="strong:G1519 lemma.TR:εις" morph="robinson:PREP">unto</w> <w src="47" lemma="strong:G2993 lemma.TR:λαοδικειαν" morph="robinson:N-ASF">Laodicea</w>.</q><verse eID="Rev.1.11"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Rev.1.11" sID="Rev.1.11"/><w>Saying</w>, <q who="Jesus" marker=""><w>I</w> <w>am</w> <w>Alpha</w> <w>and</w> <w>Omega</w>, <w>the first</w> <w>and</w> <w>the last</w>:</q> <w>and</w>, <q who="Jesus" marker=""><w>What</w> <w>thou seest</w>, <w>write</w> <w>in</w> <w>a book</w>, <w>and</w> <w>send</w> <transChange type="added">it</transChange> <w>unto the</w> <w>seven</w> <w>churches</w> <w>which are</w> <w>in</w> <w>Asia</w>; <w>unto</w> <w>Ephesus</w>, <w>and</w> <w>unto</w> <w>Smyrna</w>, <w>and</w> <w>unto</w> <w>Pergamos</w>, <w>and</w> <w>unto</w> <w>Thyatira</w>, <w>and</w> <w>unto</w> <w>Sardis</w>, <w>and</w> <w>unto</w> <w>Philadelphia</w>, <w>and</w> <w>unto</w> <w>Laodicea</w>.</q><verse eID="Rev.1.11"/>
```

### REV.1.17 (OSIS `Rev.1.17`)

- offset 27218210, length 2537, excerpt SHA-256 `9efe7586b984fe39473eabdd3df74e366fe863e04d331e8d84540d1f278082c2`
- `q` open at verse start: no
- `q` open at verse end: **yes** (continues into the next verse)
- `<q>` start tags in excerpt: 1; `</q>` end tags in excerpt: 0

```xml
<verse osisID="Rev.1.17" sID="Rev.1.17"/><w src="1" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">And</w> <w src="2" lemma="strong:G3753 lemma.TR:οτε" morph="robinson:ADV">when</w> <w src="3" lemma="strong:G3708 lemma.TR:ειδον" morph="robinson:V-2AAI-1S">I saw</w> <w src="4" lemma="strong:G846 lemma.TR:αυτον" morph="robinson:P-ASM">him</w>, <w src="5" lemma="strong:G4098 lemma.TR:επεσα" morph="robinson:V-2AAI-1S">I fell</w> <w src="6" lemma="strong:G4314 lemma.TR:προς" morph="robinson:PREP">at</w> <w src="9" lemma="strong:G846 lemma.TR:αυτου" morph="robinson:P-GSM">his</w> <w src="7 8" lemma="strong:G3588 strong:G4228 lemma.TR:τους lemma.TR:ποδας" morph="robinson:T-APM robinson:N-APM">feet</w> <w src="10" lemma="strong:G5613 lemma.TR:ως" morph="robinson:ADV">as</w> <w src="11" lemma="strong:G3498 lemma.TR:νεκρος" morph="robinson:A-NSM">dead</w>. <w src="12" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">And</w> <w src="13" lemma="strong:G2007 lemma.TR:επεθηκεν" morph="robinson:V-AAI-3S">he laid</w> <w src="16" lemma="strong:G846 lemma.TR:αυτου" morph="robinson:P-GSM">his</w> <w src="15" lemma="strong:G1188 lemma.TR:δεξιαν" morph="robinson:A-ASF">right</w> <w src="17" lemma="strong:G5495 lemma.TR:χειρα" morph="robinson:N-ASF">hand</w> <w src="18" lemma="strong:G1909 lemma.TR:επ" morph="robinson:PREP">upon</w> <w src="19" lemma="strong:G1473 lemma.TR:εμε" morph="robinson:P-1AS">me</w>, <w src="20" lemma="strong:G3004 lemma.TR:λεγων" morph="robinson:V-PAP-NSM">saying</w> <w src="21" lemma="strong:G1473 lemma.TR:μοι" morph="robinson:P-1DS">unto me</w>, <q who="Jesus" marker=""><w src="23" lemma="strong:G5399 lemma.TR:φοβου" morph="robinson:V-PNM-2S">Fear</w> <w src="22" lemma="strong:G3361 lemma.TR:μη" morph="robinson:PRT-N">not</w>; <w src="24" lemma="strong:G1473 lemma.TR:εγω" morph="robinson:P-1NS">I</w> <w src="25" lemma="strong:G1510 lemma.TR:ειμι" morph="robinson:V-PAI-1S">am</w> <w src="27" lemma="strong:G4413 lemma.TR:πρωτος" morph="robinson:A-NSM-S">the first</w> <w src="28" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="30" lemma="strong:G2078 lemma.TR:εσχατος" morph="robinson:A-NSM-S">the last</w>:<w src="14" lemma="strong:G3588 lemma.TR:την" morph="robinson:T-ASF"/><w src="26" lemma="strong:G3588 lemma.TR:ο" morph="robinson:T-NSM"/><w src="29" lemma="strong:G3588 lemma.TR:ο" morph="robinson:T-NSM"/><verse eID="Rev.1.17"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Rev.1.17" sID="Rev.1.17"/><w>And</w> <w>when</w> <w>I saw</w> <w>him</w>, <w>I fell</w> <w>at</w> <w>his</w> <w>feet</w> <w>as</w> <w>dead</w>. <w>And</w> <w>he laid</w> <w>his</w> <w>right</w> <w>hand</w> <w>upon</w> <w>me</w>, <w>saying</w> <w>unto me</w>, <q who="Jesus" marker=""><w>Fear</w> <w>not</w>; <w>I</w> <w>am</w> <w>the first</w> <w>and</w> <w>the last</w>:<w/><w/><w/><verse eID="Rev.1.17"/>
```

### REV.1.18 (OSIS `Rev.1.18`)

- offset 27220748, length 2014, excerpt SHA-256 `48021fa5bb6633aebc9a3ae99eec559a46c343fa15fa48c6f2a006731e18b379`
- `q` open at verse start: **yes**, its `<q>` tag is in `Rev.1.17` at byte 27219891
- `q` open at verse end: **yes** (continues into the next verse)
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="Rev.1.18" sID="Rev.1.18"/><transChange type="added">I am</transChange> <w src="3" lemma="strong:G2198 lemma.TR:ζων" morph="robinson:V-PAP-NSM">he that liveth</w>, <w src="4" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="5" lemma="strong:G1096 lemma.TR:εγενομην" morph="robinson:V-2ADI-1S">was</w> <w src="6" lemma="strong:G3498 lemma.TR:νεκρος" morph="robinson:A-NSM">dead</w>; <w src="7" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w>, <w src="8" lemma="strong:G3708 lemma.TR:ιδου" morph="robinson:V-2AMM-2S">behold</w>, <w src="10" lemma="strong:G1510 lemma.TR:ειμι" morph="robinson:V-PAI-1S">I am</w> <w src="9" lemma="strong:G2198 lemma.TR:ζων" morph="robinson:V-PAP-NSM">alive</w> <w src="11" lemma="strong:G1519 lemma.TR:εις" morph="robinson:PREP">for</w> <w src="12 13" lemma="strong:G3588 strong:G165 lemma.TR:τους lemma.TR:αιωνας" morph="robinson:T-APM robinson:N-APM">evermore</w>, <w src="14 15" lemma="strong:G3588 strong:G165 lemma.TR:των lemma.TR:αιωνων" morph="robinson:T-GPM robinson:N-GPM"/><w src="16" lemma="strong:G281 lemma.TR:αμην" morph="robinson:HEB">Amen</w>; <w src="17" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="18" lemma="strong:G2192 lemma.TR:εχω" morph="robinson:V-PAI-1S">have</w> <w src="19 20" lemma="strong:G3588 strong:G2807 lemma.TR:τας lemma.TR:κλεις" morph="robinson:T-APF robinson:N-APF">the keys</w> <w src="21 22" lemma="strong:G3588 strong:G86 lemma.TR:του lemma.TR:αδου" morph="robinson:T-GSM robinson:N-GSM">of hell</w> <w src="23" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="24 25" lemma="strong:G3588 strong:G2288 lemma.TR:του lemma.TR:θανατου" morph="robinson:T-GSM robinson:N-GSM">of death</w>.<w src="1" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ"/><w src="2" lemma="strong:G3588 lemma.TR:ο" morph="robinson:T-NSM"/><verse eID="Rev.1.18"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Rev.1.18" sID="Rev.1.18"/><transChange type="added">I am</transChange> <w>he that liveth</w>, <w>and</w> <w>was</w> <w>dead</w>; <w>and</w>, <w>behold</w>, <w>I am</w> <w>alive</w> <w>for</w> <w>evermore</w>, <w/><w>Amen</w>; <w>and</w> <w>have</w> <w>the keys</w> <w>of hell</w> <w>and</w> <w>of death</w>.<w/><w/><verse eID="Rev.1.18"/>
```

### REV.1.19 (OSIS `Rev.1.19`)

- offset 27222763, length 1103, excerpt SHA-256 `6a3630930ecb03e04d90cba0f1232f6e0db2714a1d7aa81c33003cd48e30b264`
- `q` open at verse start: **yes**, its `<q>` tag is in `Rev.1.17` at byte 27219891
- `q` open at verse end: **yes** (continues into the next verse)
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="Rev.1.19" sID="Rev.1.19"/><w src="1" lemma="strong:G1125 lemma.TR:γραψον" morph="robinson:V-AAM-2S">Write</w> <w src="2" lemma="strong:G3739 lemma.TR:α" morph="robinson:R-APN">the things which</w> <w src="3" lemma="strong:G3708 lemma.TR:ειδες" morph="robinson:V-2AAI-2S">thou hast seen</w>, <w src="4" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="5" lemma="strong:G3739 lemma.TR:α" morph="robinson:R-NPN">the things which</w> <w src="6" lemma="strong:G1510 lemma.TR:εισιν" morph="robinson:V-PAI-3P">are</w>, <w src="7" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="8" lemma="strong:G3739 lemma.TR:α" morph="robinson:R-NPN">the things which</w> <w src="9" lemma="strong:G3195 lemma.TR:μελλει" morph="robinson:V-PAI-3S">shall</w> <w src="10" lemma="strong:G1096 lemma.TR:γινεσθαι" morph="robinson:V-PNN">be</w> <w src="11" lemma="strong:G3326 lemma.TR:μετα" morph="robinson:PREP">hereafter</w>;<w src="12" lemma="strong:G3778 lemma.TR:ταυτα" morph="robinson:D-APN"/><verse eID="Rev.1.19"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Rev.1.19" sID="Rev.1.19"/><w>Write</w> <w>the things which</w> <w>thou hast seen</w>, <w>and</w> <w>the things which</w> <w>are</w>, <w>and</w> <w>the things which</w> <w>shall</w> <w>be</w> <w>hereafter</w>;<w/><verse eID="Rev.1.19"/>
```

### REV.1.20 (OSIS `Rev.1.20`)

- offset 27223867, length 3727, excerpt SHA-256 `c3e840fd4b13cb1c70fb4f5e563b46b48817a6bc61db555f9b61be897b7c037f`
- `q` open at verse start: **yes**, its `<q>` tag is in `Rev.1.17` at byte 27219891
- `q` open at verse end: no
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 1

```xml
<verse osisID="Rev.1.20" sID="Rev.1.20"/><w src="1 2" lemma="strong:G3588 strong:G3466 lemma.TR:το lemma.TR:μυστηριον" morph="robinson:T-NSN robinson:N-NSN">The mystery</w> <w src="3 5" lemma="strong:G3588 strong:G792 lemma.TR:των lemma.TR:αστερων" morph="robinson:T-GPM robinson:N-GPM" type="x-split-3742">of the</w> <w src="4" lemma="strong:G2033 lemma.TR:επτα" morph="robinson:A-NUI">seven</w> <w src="3 5" lemma="strong:G3588 strong:G792 lemma.TR:των lemma.TR:αστερων" morph="robinson:T-GPM robinson:N-GPM" type="x-split-3742">stars</w> <w src="6" lemma="strong:G3739 lemma.TR:ων" morph="robinson:R-GPM">which</w> <w src="7" lemma="strong:G3708 lemma.TR:ειδες" morph="robinson:V-2AAI-2S">thou sawest</w> <w src="8" lemma="strong:G1909 lemma.TR:επι" morph="robinson:PREP">in</w> <w src="11" lemma="strong:G1473 lemma.TR:μου" morph="robinson:P-1GS">my</w> <w src="9 10" lemma="strong:G3588 strong:G1188 lemma.TR:της lemma.TR:δεξιας" morph="robinson:T-GSF robinson:A-GSF">right hand</w>, <w src="12" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="13 15" lemma="strong:G3588 strong:G3087 lemma.TR:τας lemma.TR:λυχνιας" morph="robinson:T-APF robinson:N-APF" type="x-split-3743">the</w> <w src="14" lemma="strong:G2033 lemma.TR:επτα" morph="robinson:A-NUI">seven</w> <w src="16 17" lemma="strong:G3588 strong:G5552 lemma.TR:τας lemma.TR:χρυσας" morph="robinson:T-APF robinson:A-APF">golden</w> <w src="13 15" lemma="strong:G3588 strong:G3087 lemma.TR:τας lemma.TR:λυχνιας" morph="robinson:T-APF robinson:N-APF" type="x-split-3743">candlesticks</w>. <w src="18 20" lemma="strong:G3588 strong:G792 lemma.TR:οι lemma.TR:αστερες" morph="robinson:T-NPM robinson:N-NPM" type="x-split-3744">The</w> <w src="19" lemma="strong:G2033 lemma.TR:επτα" morph="robinson:A-NUI">seven</w> <w src="18 20" lemma="strong:G3588 strong:G792 lemma.TR:οι lemma.TR:αστερες" morph="robinson:T-NPM robinson:N-NPM" type="x-split-3744">stars</w> <w src="25" lemma="strong:G1510 lemma.TR:εισιν" morph="robinson:V-PAI-3P">are</w> <w src="21" lemma="strong:G32 lemma.TR:αγγελοι" morph="robinson:N-NPM">the angels</w> <w src="22 24" lemma="strong:G3588 strong:G1577 lemma.TR:των lemma.TR:εκκλησιων" morph="robinson:T-GPF robinson:N-GPF" type="x-split-3745">of the</w> <w src="23" lemma="strong:G2033 lemma.TR:επτα" morph="robinson:A-NUI">seven</w> <w src="22 24" lemma="strong:G3588 strong:G1577 lemma.TR:των lemma.TR:εκκλησιων" morph="robinson:T-GPF robinson:N-GPF" type="x-split-3745">churches</w>: <w src="26" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="27 29" lemma="strong:G3588 strong:G3087 lemma.TR:αι lemma.TR:λυχνιαι" morph="robinson:T-NPF robinson:N-NPF" type="x-split-3746">the</w> <w src="28" lemma="strong:G2033 lemma.TR:επτα" morph="robinson:A-NUI">seven</w> <w src="27 29" lemma="strong:G3588 strong:G3087 lemma.TR:αι lemma.TR:λυχνιαι" morph="robinson:T-NPF robinson:N-NPF" type="x-split-3746">candlesticks</w> <w src="30" lemma="strong:G3739 lemma.TR:ας" morph="robinson:R-APF">which</w> <w src="31" lemma="strong:G3708 lemma.TR:ειδες" morph="robinson:V-2AAI-2S">thou sawest</w> <w src="34" lemma="strong:G1510 lemma.TR:εισιν" morph="robinson:V-PAI-3P">are</w> <w src="33" lemma="strong:G1577 lemma.TR:εκκλησιαι" morph="robinson:N-NPF" type="x-split-3747">the</w> <w src="32" lemma="strong:G2033 lemma.TR:επτα" morph="robinson:A-NUI">seven</w> <w src="33" lemma="strong:G1577 lemma.TR:εκκλησιαι" morph="robinson:N-NPF" type="x-split-3747">churches</w>.</q><verse eID="Rev.1.20"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Rev.1.20" sID="Rev.1.20"/><w>The mystery</w> <w>of the</w> <w>seven</w> <w>stars</w> <w>which</w> <w>thou sawest</w> <w>in</w> <w>my</w> <w>right hand</w>, <w>and</w> <w>the</w> <w>seven</w> <w>golden</w> <w>candlesticks</w>. <w>The</w> <w>seven</w> <w>stars</w> <w>are</w> <w>the angels</w> <w>of the</w> <w>seven</w> <w>churches</w>: <w>and</w> <w>the</w> <w>seven</w> <w>candlesticks</w> <w>which</w> <w>thou sawest</w> <w>are</w> <w>the</w> <w>seven</w> <w>churches</w>.</q><verse eID="Rev.1.20"/>
```

### REV.2.1 (OSIS `Rev.2.1`)

- offset 27227698, length 2453, excerpt SHA-256 `abe15ceb042763162816f379e8b5282ed532ceae5c5cdd1d7cffe7353edd2b77`
- `q` open at verse start: no
- `q` open at verse end: **yes** (continues into the next verse)
- `<q>` start tags in excerpt: 1; `</q>` end tags in excerpt: 0

```xml
<verse osisID="Rev.2.1" sID="Rev.2.1"/><q who="Jesus" marker=""><w src="1 2" lemma="strong:G3588 strong:G32 lemma.TR:τω lemma.TR:αγγελω" morph="robinson:T-DSM robinson:N-DSM">Unto the angel</w> <w src="5" lemma="strong:G1577 lemma.TR:εκκλησιας" morph="robinson:N-GSF">of the church</w> <w src="3 4" lemma="strong:G3588 strong:G2179 lemma.TR:της lemma.TR:εφεσινης" morph="robinson:T-GSF robinson:N-GSF">of Ephesus</w> <w src="6" lemma="strong:G1125 lemma.TR:γραψον" morph="robinson:V-AAM-2S">write</w>; <w src="7" lemma="strong:G3592 lemma.TR:ταδε" morph="robinson:D-APN">These things</w> <w src="8" lemma="strong:G3004 lemma.TR:λεγει" morph="robinson:V-PAI-3S">saith</w> <w src="9 10" lemma="strong:G3588 strong:G2902 lemma.TR:ο lemma.TR:κρατων" morph="robinson:T-NSM robinson:V-PAP-NSM">he that holdeth</w> <w src="11 13" lemma="strong:G3588 strong:G792 lemma.TR:τους lemma.TR:αστερας" morph="robinson:T-APM robinson:N-APM" type="x-split-3748">the</w> <w src="12" lemma="strong:G2033 lemma.TR:επτα" morph="robinson:A-NUI">seven</w> <w src="11 13" lemma="strong:G3588 strong:G792 lemma.TR:τους lemma.TR:αστερας" morph="robinson:T-APM robinson:N-APM" type="x-split-3748">stars</w> <w src="14" lemma="strong:G1722 lemma.TR:εν" morph="robinson:PREP">in</w> <w src="17" lemma="strong:G846 lemma.TR:αυτου" morph="robinson:P-GSM">his</w> <w src="15 16" lemma="strong:G3588 strong:G1188 lemma.TR:τη lemma.TR:δεξια" morph="robinson:T-DSF robinson:A-DSF">right hand</w>, <w src="18 19" lemma="strong:G3588 strong:G4043 lemma.TR:ο lemma.TR:περιπατων" morph="robinson:T-NSM robinson:V-PAP-NSM">who walketh</w> <w src="20" lemma="strong:G1722 lemma.TR:εν" morph="robinson:PREP">in</w> <w src="21" lemma="strong:G3319 lemma.TR:μεσω" morph="robinson:A-DSN">the midst</w> <w src="22 24" lemma="strong:G3588 strong:G3087 lemma.TR:των lemma.TR:λυχνιων" morph="robinson:T-GPF robinson:N-GPF" type="x-split-3749">of the</w> <w src="23" lemma="strong:G2033 lemma.TR:επτα" morph="robinson:A-NUI">seven</w> <w src="26" lemma="strong:G5552 lemma.TR:χρυσων" morph="robinson:A-GPF">golden</w> <w src="22 24" lemma="strong:G3588 strong:G3087 lemma.TR:των lemma.TR:λυχνιων" morph="robinson:T-GPF robinson:N-GPF" type="x-split-3749">candlesticks</w>;<w src="25" lemma="strong:G3588 lemma.TR:των" morph="robinson:T-GPF"/><verse eID="Rev.2.1"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Rev.2.1" sID="Rev.2.1"/><q who="Jesus" marker=""><w>Unto the angel</w> <w>of the church</w> <w>of Ephesus</w> <w>write</w>; <w>These things</w> <w>saith</w> <w>he that holdeth</w> <w>the</w> <w>seven</w> <w>stars</w> <w>in</w> <w>his</w> <w>right hand</w>, <w>who walketh</w> <w>in</w> <w>the midst</w> <w>of the</w> <w>seven</w> <w>golden</w> <w>candlesticks</w>;<w/><verse eID="Rev.2.1"/>
```

### REV.3.6 (OSIS `Rev.3.6`)

- offset 27296097, length 864, excerpt SHA-256 `24964ee4c602183dfb21c9d4bfa2ecbf6517fd28883a2b09cbdb911864e96dde`
- `q` open at verse start: **yes**, its `<q>` tag is in `Rev.3.1` at byte 27283936
- `q` open at verse end: **yes** (continues into the next verse)
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="Rev.3.6" sID="Rev.3.6"/><w src="2" lemma="strong:G2192 lemma.TR:εχων" morph="robinson:V-PAP-NSM">He that hath</w> <w src="3" lemma="strong:G3775 lemma.TR:ους" morph="robinson:N-ASN">an ear</w>, <w src="4" lemma="strong:G191 lemma.TR:ακουσατω" morph="robinson:V-AAM-3S">let him hear</w> <w src="5" lemma="strong:G5101 lemma.TR:τι" morph="robinson:I-ASN">what</w> <w src="6 7" lemma="strong:G3588 strong:G4151 lemma.TR:το lemma.TR:πνευμα" morph="robinson:T-NSN robinson:N-NSN">the Spirit</w> <w src="8" lemma="strong:G3004 lemma.TR:λεγει" morph="robinson:V-PAI-3S">saith</w> <w src="9 10" lemma="strong:G3588 strong:G1577 lemma.TR:ταις lemma.TR:εκκλησιαις" morph="robinson:T-DPF robinson:N-DPF">unto the churches</w>.<w src="1" lemma="strong:G3588 lemma.TR:ο" morph="robinson:T-NSM"/><verse eID="Rev.3.6"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Rev.3.6" sID="Rev.3.6"/><w>He that hath</w> <w>an ear</w>, <w>let him hear</w> <w>what</w> <w>the Spirit</w> <w>saith</w> <w>unto the churches</w>.<w/><verse eID="Rev.3.6"/>
```

### REV.3.20 (OSIS `Rev.3.20`)

- offset 27325580, length 2330, excerpt SHA-256 `3b5ab3474ff180ca722181ab5d4ad259b4d558600c0f55f71d1f741b37ab881d`
- `q` open at verse start: **yes**, its `<q>` tag is in `Rev.3.1` at byte 27283936
- `q` open at verse end: **yes** (continues into the next verse)
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="Rev.3.20" sID="Rev.3.20"/><w src="1" lemma="strong:G3708 lemma.TR:ιδου" morph="robinson:V-2AMM-2S">Behold</w>, <w src="2" lemma="strong:G2476 lemma.TR:εστηκα" morph="robinson:V-RAI-1S">I stand</w> <w src="3" lemma="strong:G1909 lemma.TR:επι" morph="robinson:PREP">at</w> <w src="4 5" lemma="strong:G3588 strong:G2374 lemma.TR:την lemma.TR:θυραν" morph="robinson:T-ASF robinson:N-ASF">the door</w>, <w src="6" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="7" lemma="strong:G2925 lemma.TR:κρουω" morph="robinson:V-PAI-1S">knock</w>: <w src="8" lemma="strong:G1437 lemma.TR:εαν" morph="robinson:COND">if</w> <w src="9" lemma="strong:G5100 lemma.TR:τις" morph="robinson:X-NSM">any man</w> <w src="10" lemma="strong:G191 lemma.TR:ακουση" morph="robinson:V-AAS-3S">hear</w> <w src="13" lemma="strong:G1473 lemma.TR:μου" morph="robinson:P-1GS">my</w> <w src="11 12" lemma="strong:G3588 strong:G5456 lemma.TR:της lemma.TR:φωνης" morph="robinson:T-GSF robinson:N-GSF">voice</w>, <w src="14" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="15" lemma="strong:G455 lemma.TR:ανοιξη" morph="robinson:V-AAS-3S">open</w> <w src="16 17" lemma="strong:G3588 strong:G2374 lemma.TR:την lemma.TR:θυραν" morph="robinson:T-ASF robinson:N-ASF">the door</w>, <w src="18" lemma="strong:G1525 lemma.TR:εισελευσομαι" morph="robinson:V-FDI-1S">I will come in</w> <w src="19" lemma="strong:G4314 lemma.TR:προς" morph="robinson:PREP">to</w> <w src="20" lemma="strong:G846 lemma.TR:αυτον" morph="robinson:P-ASM">him</w>, <w src="21" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="22" lemma="strong:G1172 lemma.TR:δειπνησω" morph="robinson:V-FAI-1S">will sup</w> <w src="23" lemma="strong:G3326 lemma.TR:μετ" morph="robinson:PREP">with</w> <w src="24" lemma="strong:G846 lemma.TR:αυτου" morph="robinson:P-GSM">him</w>, <w src="25" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="26" lemma="strong:G846 lemma.TR:αυτος" morph="robinson:P-NSM">he</w> <w src="27" lemma="strong:G3326 lemma.TR:μετ" morph="robinson:PREP">with</w> <w src="28" lemma="strong:G1473 lemma.TR:εμου" morph="robinson:P-1GS">me</w>.<verse eID="Rev.3.20"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Rev.3.20" sID="Rev.3.20"/><w>Behold</w>, <w>I stand</w> <w>at</w> <w>the door</w>, <w>and</w> <w>knock</w>: <w>if</w> <w>any man</w> <w>hear</w> <w>my</w> <w>voice</w>, <w>and</w> <w>open</w> <w>the door</w>, <w>I will come in</w> <w>to</w> <w>him</w>, <w>and</w> <w>will sup</w> <w>with</w> <w>him</w>, <w>and</w> <w>he</w> <w>with</w> <w>me</w>.<verse eID="Rev.3.20"/>
```

### REV.3.22 (OSIS `Rev.3.22`)

- offset 27329884, length 871, excerpt SHA-256 `f78cd2cede41d4af0844373c9db9c5de028ad2825db1071129c25b3cb3690638`
- `q` open at verse start: **yes**, its `<q>` tag is in `Rev.3.1` at byte 27283936
- `q` open at verse end: no
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 1

```xml
<verse osisID="Rev.3.22" sID="Rev.3.22"/><w src="2" lemma="strong:G2192 lemma.TR:εχων" morph="robinson:V-PAP-NSM">He that hath</w> <w src="3" lemma="strong:G3775 lemma.TR:ους" morph="robinson:N-ASN">an ear</w>, <w src="4" lemma="strong:G191 lemma.TR:ακουσατω" morph="robinson:V-AAM-3S">let him hear</w> <w src="5" lemma="strong:G5101 lemma.TR:τι" morph="robinson:I-ASN">what</w> <w src="6 7" lemma="strong:G3588 strong:G4151 lemma.TR:το lemma.TR:πνευμα" morph="robinson:T-NSN robinson:N-NSN">the Spirit</w> <w src="8" lemma="strong:G3004 lemma.TR:λεγει" morph="robinson:V-PAI-3S">saith</w> <w src="9 10" lemma="strong:G3588 strong:G1577 lemma.TR:ταις lemma.TR:εκκλησιαις" morph="robinson:T-DPF robinson:N-DPF">unto the churches</w>.</q><w src="1" lemma="strong:G3588 lemma.TR:ο" morph="robinson:T-NSM"/><verse eID="Rev.3.22"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Rev.3.22" sID="Rev.3.22"/><w>He that hath</w> <w>an ear</w>, <w>let him hear</w> <w>what</w> <w>the Spirit</w> <w>saith</w> <w>unto the churches</w>.</q><w/><verse eID="Rev.3.22"/>
```

### REV.4.1 (OSIS `Rev.4.1`)

- offset 27330859, length 2903, excerpt SHA-256 `32517f3553735bf87b7a3a3d3308a461f190b298078892505bbbb071b2b084fa`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="Rev.4.1" sID="Rev.4.1"/><w src="1" lemma="strong:G3326 lemma.TR:μετα" morph="robinson:PREP">After</w> <w src="2" lemma="strong:G3778 lemma.TR:ταυτα" morph="robinson:D-APN">this</w> <w src="3" lemma="strong:G3708 lemma.TR:ειδον" morph="robinson:V-2AAI-1S">I looked</w>, <w src="4" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w>, <w src="5" lemma="strong:G3708 lemma.TR:ιδου" morph="robinson:V-2AMM-2S">behold</w>, <w src="6" lemma="strong:G2374 lemma.TR:θυρα" morph="robinson:N-NSF">a door</w> <transChange type="added">was</transChange> <w src="7" lemma="strong:G455 lemma.TR:ηνεωγμενη" morph="robinson:V-RPP-NSF">opened</w> <w src="8" lemma="strong:G1722 lemma.TR:εν" morph="robinson:PREP">in</w> <w src="9 10" lemma="strong:G3588 strong:G3772 lemma.TR:τω lemma.TR:ουρανω" morph="robinson:T-DSM robinson:N-DSM">heaven</w>: <w src="11" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="15" lemma="strong:G4413 lemma.TR:πρωτη" morph="robinson:A-NSF-S">the first</w> <w src="12 13" lemma="strong:G3588 strong:G5456 lemma.TR:η lemma.TR:φωνη" morph="robinson:T-NSF robinson:N-NSF">voice</w> <w src="16" lemma="strong:G3739 lemma.TR:ην" morph="robinson:R-ASF">which</w> <w src="17" lemma="strong:G191 lemma.TR:ηκουσα" morph="robinson:V-AAI-1S">I heard</w> <transChange type="added">was</transChange> <w src="18" lemma="strong:G5613 lemma.TR:ως" morph="robinson:ADV">as it were</w> <w src="19" lemma="strong:G4536 lemma.TR:σαλπιγγος" morph="robinson:N-GSF">of a trumpet</w> <w src="20" lemma="strong:G2980 lemma.TR:λαλουσης" morph="robinson:V-PAP-GSF">talking</w> <w src="21" lemma="strong:G3326 lemma.TR:μετ" morph="robinson:PREP">with</w> <w src="22" lemma="strong:G1473 lemma.TR:εμου" morph="robinson:P-1GS">me</w>; <w src="23" lemma="strong:G3004 lemma.TR:λεγουσα" morph="robinson:V-PAP-NSF">which said</w>, <w src="24" lemma="strong:G305 lemma.TR:αναβα" morph="robinson:V-2AAM-2S">Come up</w> <w src="25" lemma="strong:G5602 lemma.TR:ωδε" morph="robinson:ADV">hither</w>, <w src="26" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="27" lemma="strong:G1166 lemma.TR:δειξω" morph="robinson:V-FAI-1S">I will shew</w> <w src="28" lemma="strong:G4771 lemma.TR:σοι" morph="robinson:P-2DS">thee</w> <w src="33" lemma="strong:G3778 lemma.TR:ταυτα" morph="robinson:D-APN">things</w> <w src="29" lemma="strong:G3739 lemma.TR:α" morph="robinson:R-APN">which</w> <w src="30" lemma="strong:G1163 lemma.TR:δει" morph="robinson:V-PAI-3S">must</w> <w src="31" lemma="strong:G1096 lemma.TR:γενεσθαι" morph="robinson:V-2ADN">be</w> <w src="32" lemma="strong:G3326 lemma.TR:μετα" morph="robinson:PREP">hereafter</w>.<w src="14" lemma="strong:G3588 lemma.TR:η" morph="robinson:T-NSF"/><verse eID="Rev.4.1"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Rev.4.1" sID="Rev.4.1"/><w>After</w> <w>this</w> <w>I looked</w>, <w>and</w>, <w>behold</w>, <w>a door</w> <transChange type="added">was</transChange> <w>opened</w> <w>in</w> <w>heaven</w>: <w>and</w> <w>the first</w> <w>voice</w> <w>which</w> <w>I heard</w> <transChange type="added">was</transChange> <w>as it were</w> <w>of a trumpet</w> <w>talking</w> <w>with</w> <w>me</w>; <w>which said</w>, <w>Come up</w> <w>hither</w>, <w>and</w> <w>I will shew</w> <w>thee</w> <w>things</w> <w>which</w> <w>must</w> <w>be</w> <w>hereafter</w>.<w/><verse eID="Rev.4.1"/>
```

### REV.22.7 (OSIS `Rev.22.7`)

- offset 28016757, length 1183, excerpt SHA-256 `8bbb49fc5a17f8b8642d6559d2f2ee690925b20533df531dcbcffafddbaeb51b`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 1; `</q>` end tags in excerpt: 1

```xml
<verse osisID="Rev.22.7" sID="Rev.22.7"/><q who="Jesus" marker=""><w src="1" lemma="strong:G3708 lemma.TR:ιδου" morph="robinson:V-2AMM-2S">Behold</w>, <w src="2" lemma="strong:G2064 lemma.TR:ερχομαι" morph="robinson:V-PNI-1S">I come</w> <w src="3" lemma="strong:G5035 lemma.TR:ταχυ" morph="robinson:ADV">quickly</w>:</q> <w src="4" lemma="strong:G3107 lemma.TR:μακαριος" morph="robinson:A-NSM">blessed</w> <transChange type="added">is</transChange> <w src="5" lemma="strong:G3588 lemma.TR:ο" morph="robinson:T-NSM">he</w> <w src="6" lemma="strong:G5083 lemma.TR:τηρων" morph="robinson:V-PAP-NSM">that keepeth</w> <w src="7 8" lemma="strong:G3588 strong:G3056 lemma.TR:τους lemma.TR:λογους" morph="robinson:T-APM robinson:N-APM">the sayings</w> <w src="9 10" lemma="strong:G3588 strong:G4394 lemma.TR:της lemma.TR:προφητειας" morph="robinson:T-GSF robinson:N-GSF">of the prophecy</w> <w src="13" lemma="strong:G3778 lemma.TR:τουτου" morph="robinson:D-GSN">of this</w> <w src="11 12" lemma="strong:G3588 strong:G975 lemma.TR:του lemma.TR:βιβλιου" morph="robinson:T-GSN robinson:N-GSN">book</w>.<verse eID="Rev.22.7"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Rev.22.7" sID="Rev.22.7"/><q who="Jesus" marker=""><w>Behold</w>, <w>I come</w> <w>quickly</w>:</q> <w>blessed</w> <transChange type="added">is</transChange> <w>he</w> <w>that keepeth</w> <w>the sayings</w> <w>of the prophecy</w> <w>of this</w> <w>book</w>.<verse eID="Rev.22.7"/>
```

### REV.22.11 (OSIS `Rev.22.11`)

- offset 28023639, length 1631, excerpt SHA-256 `a54b476a6bf700467ef20c907e770bd808785456785375da7d745e1587ff75ef`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="Rev.22.11" sID="Rev.22.11"/><w src="1 2" lemma="strong:G3588 strong:G91 lemma.TR:ο lemma.TR:αδικων" morph="robinson:T-NSM robinson:V-PAP-NSM">He that is unjust</w>, <w src="3" lemma="strong:G91 lemma.TR:αδικησατω" morph="robinson:V-AAM-3S">let him be unjust</w> <w src="4" lemma="strong:G2089 lemma.TR:ετι" morph="robinson:ADV">still</w>: <w src="5" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="6 7" lemma="strong:G3588 strong:G4510 lemma.TR:ο lemma.TR:ρυπων" morph="robinson:T-NSM robinson:V-PAP-NSM">he which is filthy</w>, <w src="8" lemma="strong:G4510 lemma.TR:ρυπωσατω" morph="robinson:V-AAM-3S">let him be filthy</w> <w src="9" lemma="strong:G2089 lemma.TR:ετι" morph="robinson:ADV">still</w>: <w src="10" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="11 12" lemma="strong:G3588 strong:G1342 lemma.TR:ο lemma.TR:δικαιος" morph="robinson:T-NSM robinson:A-NSM">he that is righteous</w>, <w src="13" lemma="strong:G1344 lemma.TR:δικαιωθητω" morph="robinson:V-APM-3S">let him be righteous</w> <w src="14" lemma="strong:G2089 lemma.TR:ετι" morph="robinson:ADV">still</w>: <w src="15" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="16 17" lemma="strong:G3588 strong:G40 lemma.TR:ο lemma.TR:αγιος" morph="robinson:T-NSM robinson:A-NSM">he that is holy</w>, <w src="18" lemma="strong:G37 lemma.TR:αγιασθητω" morph="robinson:V-APM-3S">let him be holy</w> <w src="19" lemma="strong:G2089 lemma.TR:ετι" morph="robinson:ADV">still</w>.<verse eID="Rev.22.11"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Rev.22.11" sID="Rev.22.11"/><w>He that is unjust</w>, <w>let him be unjust</w> <w>still</w>: <w>and</w> <w>he which is filthy</w>, <w>let him be filthy</w> <w>still</w>: <w>and</w> <w>he that is righteous</w>, <w>let him be righteous</w> <w>still</w>: <w>and</w> <w>he that is holy</w>, <w>let him be holy</w> <w>still</w>.<verse eID="Rev.22.11"/>
```

### REV.22.12 (OSIS `Rev.22.12`)

- offset 28025271, length 1505, excerpt SHA-256 `239876d12ff682cfe8e09e5fe18bead7cefebfb9fe4678d4b063811428581a0b`
- `q` open at verse start: no
- `q` open at verse end: **yes** (continues into the next verse)
- `<q>` start tags in excerpt: 1; `</q>` end tags in excerpt: 0

```xml
<verse osisID="Rev.22.12" sID="Rev.22.12"/><q who="Jesus" marker=""><w src="1" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">And</w>, <w src="2" lemma="strong:G3708 lemma.TR:ιδου" morph="robinson:V-2AMM-2S">behold</w>, <w src="3" lemma="strong:G2064 lemma.TR:ερχομαι" morph="robinson:V-PNI-1S">I come</w> <w src="4" lemma="strong:G5035 lemma.TR:ταχυ" morph="robinson:ADV">quickly</w>; <w src="5" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="8" lemma="strong:G1473 lemma.TR:μου" morph="robinson:P-1GS">my</w> <w src="6 7" lemma="strong:G3588 strong:G3408 lemma.TR:ο lemma.TR:μισθος" morph="robinson:T-NSM robinson:N-NSM">reward</w> <transChange type="added">is</transChange> <w src="9" lemma="strong:G3326 lemma.TR:μετ" morph="robinson:PREP">with</w> <w src="10" lemma="strong:G1473 lemma.TR:εμου" morph="robinson:P-1GS">me</w>, <w src="11" lemma="strong:G591 lemma.TR:αποδουναι" morph="robinson:V-2AAN">to give</w> <w src="12" lemma="strong:G1538 lemma.TR:εκαστω" morph="robinson:A-DSM">every man</w> <w src="13" lemma="strong:G5613 lemma.TR:ως" morph="robinson:ADV">according as</w> <w src="16" lemma="strong:G846 lemma.TR:αυτου" morph="robinson:P-GSM">his</w> <w src="14 15" lemma="strong:G3588 strong:G2041 lemma.TR:το lemma.TR:εργον" morph="robinson:T-NSN robinson:N-NSN">work</w> <w src="17" lemma="strong:G1510 lemma.TR:εσται" morph="robinson:V-FDI-3S">shall be</w>.<verse eID="Rev.22.12"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Rev.22.12" sID="Rev.22.12"/><q who="Jesus" marker=""><w>And</w>, <w>behold</w>, <w>I come</w> <w>quickly</w>; <w>and</w> <w>my</w> <w>reward</w> <transChange type="added">is</transChange> <w>with</w> <w>me</w>, <w>to give</w> <w>every man</w> <w>according as</w> <w>his</w> <w>work</w> <w>shall be</w>.<verse eID="Rev.22.12"/>
```

### REV.22.13 (OSIS `Rev.22.13`)

- offset 28026777, length 1158, excerpt SHA-256 `8df0c8dc464ef30022ec8a1a49116a2c6af2de3fb078e026b1d6dec43a3d99e9`
- `q` open at verse start: **yes**, its `<q>` tag is in `Rev.22.12` at byte 28025314
- `q` open at verse end: no
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 1

```xml
<verse osisID="Rev.22.13" sID="Rev.22.13"/><w src="1" lemma="strong:G1473 lemma.TR:εγω" morph="robinson:P-1NS">I</w> <w src="2" lemma="strong:G1510 lemma.TR:ειμι" morph="robinson:V-PAI-1S">am</w> <w src="3 4" lemma="strong:G3588 strong:G1 lemma.TR:το lemma.TR:α" morph="robinson:T-NSN robinson:N-LI">Alpha</w> <w src="5" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="6 7" lemma="strong:G3588 strong:G5598 lemma.TR:το lemma.TR:ω" morph="robinson:T-NSN robinson:N-LI">Omega</w>, <w src="8" lemma="strong:G746 lemma.TR:αρχη" morph="robinson:N-NSF">the beginning</w> <w src="9" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="10" lemma="strong:G5056 lemma.TR:τελος" morph="robinson:N-NSN">the end</w>, <w src="11 12" lemma="strong:G3588 strong:G4413 lemma.TR:ο lemma.TR:πρωτος" morph="robinson:T-NSM robinson:A-NSM-S">the first</w> <w src="13" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="14 15" lemma="strong:G3588 strong:G2078 lemma.TR:ο lemma.TR:εσχατος" morph="robinson:T-NSM robinson:A-NSM-S">the last</w>.</q><verse eID="Rev.22.13"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Rev.22.13" sID="Rev.22.13"/><w>I</w> <w>am</w> <w>Alpha</w> <w>and</w> <w>Omega</w>, <w>the beginning</w> <w>and</w> <w>the end</w>, <w>the first</w> <w>and</w> <w>the last</w>.</q><verse eID="Rev.22.13"/>
```

### REV.22.14 (OSIS `Rev.22.14`)

- offset 28027936, length 1891, excerpt SHA-256 `7634570a570bec1780b308557d51a77902137500e4fdacadd8232eaeedfbb03c`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="Rev.22.14" sID="Rev.22.14"/><w src="1" lemma="strong:G3107 lemma.TR:μακαριοι" morph="robinson:A-NPM">Blessed</w> <transChange type="added">are</transChange> <w src="2" lemma="strong:G3588 lemma.TR:οι" morph="robinson:T-NPM">they</w> <w src="3" lemma="strong:G4160 lemma.TR:ποιουντες" morph="robinson:V-PAP-NPM">that do</w> <w src="6" lemma="strong:G846 lemma.TR:αυτου" morph="robinson:P-GSM">his</w> <w src="4 5" lemma="strong:G3588 strong:G1785 lemma.TR:τας lemma.TR:εντολας" morph="robinson:T-APF robinson:N-APF">commandments</w>, <w src="7" lemma="strong:G2443 lemma.TR:ινα" morph="robinson:CONJ">that</w> <w src="11" lemma="strong:G846 lemma.TR:αυτων" morph="robinson:P-GPM">they</w> <w src="8" lemma="strong:G1510 lemma.TR:εσται" morph="robinson:V-FDI-3S">may have</w> <w src="9 10" lemma="strong:G3588 strong:G1849 lemma.TR:η lemma.TR:εξουσια" morph="robinson:T-NSF robinson:N-NSF">right</w> <w src="12" lemma="strong:G1909 lemma.TR:επι" morph="robinson:PREP">to</w> <w src="13 14" lemma="strong:G3588 strong:G3586 lemma.TR:το lemma.TR:ξυλον" morph="robinson:T-ASN robinson:N-ASN">the tree</w> <w src="15 16" lemma="strong:G3588 strong:G2222 lemma.TR:της lemma.TR:ζωης" morph="robinson:T-GSF robinson:N-GSF">of life</w>, <w src="17" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="20" lemma="strong:G1525 lemma.TR:εισελθωσιν" morph="robinson:V-2AAS-3P">may enter in</w> <w src="18 19" lemma="strong:G3588 strong:G4440 lemma.TR:τοις lemma.TR:πυλωσιν" morph="robinson:T-DPM robinson:N-DPM">through the gates</w> <w src="21" lemma="strong:G1519 lemma.TR:εις" morph="robinson:PREP">into</w> <w src="22 23" lemma="strong:G3588 strong:G4172 lemma.TR:την lemma.TR:πολιν" morph="robinson:T-ASF robinson:N-ASF">the city</w>.<verse eID="Rev.22.14"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Rev.22.14" sID="Rev.22.14"/><w>Blessed</w> <transChange type="added">are</transChange> <w>they</w> <w>that do</w> <w>his</w> <w>commandments</w>, <w>that</w> <w>they</w> <w>may have</w> <w>right</w> <w>to</w> <w>the tree</w> <w>of life</w>, <w>and</w> <w>may enter in</w> <w>through the gates</w> <w>into</w> <w>the city</w>.<verse eID="Rev.22.14"/>
```

### REV.22.15 (OSIS `Rev.22.15`)

- offset 28029828, length 1836, excerpt SHA-256 `b15e5faaeceaa100047e5331b332e824dc390fc59653e68442d74cf21c4d1669`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="Rev.22.15" sID="Rev.22.15"/><w src="2" lemma="strong:G1161 lemma.TR:δε" morph="robinson:CONJ">For</w> <w src="1" lemma="strong:G1854 lemma.TR:εξω" morph="robinson:ADV">without</w> <transChange type="added">are</transChange> <w src="3 4" lemma="strong:G3588 strong:G2965 lemma.TR:οι lemma.TR:κυνες" morph="robinson:T-NPM robinson:N-NPM">dogs</w>, <w src="5" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="6 7" lemma="strong:G3588 strong:G5333 lemma.TR:οι lemma.TR:φαρμακοι" morph="robinson:T-NPM robinson:N-NPM">sorcerers</w>, <w src="8" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="9 10" lemma="strong:G3588 strong:G4205 lemma.TR:οι lemma.TR:πορνοι" morph="robinson:T-NPM robinson:N-NPM">whoremongers</w>, <w src="11" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="12 13" lemma="strong:G3588 strong:G5406 lemma.TR:οι lemma.TR:φονεις" morph="robinson:T-NPM robinson:N-NPM">murderers</w>, <w src="14" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="15 16" lemma="strong:G3588 strong:G1496 lemma.TR:οι lemma.TR:ειδωλολατραι" morph="robinson:T-NPM robinson:N-NPM">idolaters</w>, <w src="17" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="18" lemma="strong:G3956 lemma.TR:πας" morph="robinson:A-NSM">whosoever</w> <w src="19 20" lemma="strong:G3588 strong:G5368 lemma.TR:ο lemma.TR:φιλων" morph="robinson:T-NSM robinson:V-PAP-NSM">loveth</w> <w src="21" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="22" lemma="strong:G4160 lemma.TR:ποιων" morph="robinson:V-PAP-NSM">maketh</w> <w src="23" lemma="strong:G5579 lemma.TR:ψευδος" morph="robinson:N-ASN">a lie</w>.<verse eID="Rev.22.15"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Rev.22.15" sID="Rev.22.15"/><w>For</w> <w>without</w> <transChange type="added">are</transChange> <w>dogs</w>, <w>and</w> <w>sorcerers</w>, <w>and</w> <w>whoremongers</w>, <w>and</w> <w>murderers</w>, <w>and</w> <w>idolaters</w>, <w>and</w> <w>whosoever</w> <w>loveth</w> <w>and</w> <w>maketh</w> <w>a lie</w>.<verse eID="Rev.22.15"/>
```

### REV.22.16 (OSIS `Rev.22.16`)

- offset 28031665, length 2205, excerpt SHA-256 `0aeb106736de817959dbc8d42ca0f9eef972c4350b4d66ded3a0ae7548f53dd9`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 1; `</q>` end tags in excerpt: 1

```xml
<verse osisID="Rev.22.16" sID="Rev.22.16"/><q who="Jesus" marker=""><w src="1" lemma="strong:G1473 lemma.TR:εγω" morph="robinson:P-1NS">I</w> <w src="2" lemma="strong:G2424 lemma.TR:ιησους" morph="robinson:N-NSM">Jesus</w> <w src="3" lemma="strong:G3992 lemma.TR:επεμψα" morph="robinson:V-AAI-1S">have sent</w> <w src="6" lemma="strong:G1473 lemma.TR:μου" morph="robinson:P-1GS">mine</w> <w src="4 5" lemma="strong:G3588 strong:G32 lemma.TR:τον lemma.TR:αγγελον" morph="robinson:T-ASM robinson:N-ASM">angel</w> <w src="7" lemma="strong:G3140 lemma.TR:μαρτυρησαι" morph="robinson:V-AAN">to testify</w> <w src="8" lemma="strong:G4771 lemma.TR:υμιν" morph="robinson:P-2DP">unto you</w> <w src="9" lemma="strong:G3778 lemma.TR:ταυτα" morph="robinson:D-APN">these things</w> <w src="10" lemma="strong:G1909 lemma.TR:επι" morph="robinson:PREP">in</w> <w src="11 12" lemma="strong:G3588 strong:G1577 lemma.TR:ταις lemma.TR:εκκλησιαις" morph="robinson:T-DPF robinson:N-DPF">the churches</w>. <w src="13" lemma="strong:G1473 lemma.TR:εγω" morph="robinson:P-1NS">I</w> <w src="14" lemma="strong:G1510 lemma.TR:ειμι" morph="robinson:V-PAI-1S">am</w> <w src="15 16" lemma="strong:G3588 strong:G4491 lemma.TR:η lemma.TR:ριζα" morph="robinson:T-NSF robinson:N-NSF">the root</w> <w src="17" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="18 19" lemma="strong:G3588 strong:G1085 lemma.TR:το lemma.TR:γενος" morph="robinson:T-NSN robinson:N-NSN">the offspring</w> <w src="20 21" lemma="strong:G3588 strong:G1138 lemma.TR:του lemma.TR:δαβιδ" morph="robinson:T-GSM robinson:N-PRI">of David</w>, <transChange type="added">and</transChange> <w src="24 25" lemma="strong:G3588 strong:G2986 lemma.TR:ο lemma.TR:λαμπρος" morph="robinson:T-NSM robinson:A-NSM">the bright</w> <w src="26" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="27" lemma="strong:G3720 lemma.TR:ορθρινος" morph="robinson:A-NSM">morning</w> <w src="22 23" lemma="strong:G3588 strong:G792 lemma.TR:ο lemma.TR:αστηρ" morph="robinson:T-NSM robinson:N-NSM">star</w>.</q><verse eID="Rev.22.16"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Rev.22.16" sID="Rev.22.16"/><q who="Jesus" marker=""><w>I</w> <w>Jesus</w> <w>have sent</w> <w>mine</w> <w>angel</w> <w>to testify</w> <w>unto you</w> <w>these things</w> <w>in</w> <w>the churches</w>. <w>I</w> <w>am</w> <w>the root</w> <w>and</w> <w>the offspring</w> <w>of David</w>, <transChange type="added">and</transChange> <w>the bright</w> <w>and</w> <w>morning</w> <w>star</w>.</q><verse eID="Rev.22.16"/>
```

### REV.22.17 (OSIS `Rev.22.17`)

- offset 28033871, length 2030, excerpt SHA-256 `03c12316450007d50421eb71b44ddd80f2c7fd42fe8e3c42cec84befadaad37e`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="Rev.22.17" sID="Rev.22.17"/><w src="1" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">And</w> <w src="2 3" lemma="strong:G3588 strong:G4151 lemma.TR:το lemma.TR:πνευμα" morph="robinson:T-NSN robinson:N-NSN">the Spirit</w> <w src="4" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">and</w> <w src="5 6" lemma="strong:G3588 strong:G3565 lemma.TR:η lemma.TR:νυμφη" morph="robinson:T-NSF robinson:N-NSF">the bride</w> <w src="7" lemma="strong:G3004 lemma.TR:λεγουσιν" morph="robinson:V-PAI-3P">say</w>, <w src="8" lemma="strong:G2064 lemma.TR:ελθε" morph="robinson:V-2AAM-2S">Come</w>. <w src="9" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">And</w> <w src="10 11" lemma="strong:G3588 strong:G191 lemma.TR:ο lemma.TR:ακουων" morph="robinson:T-NSM robinson:V-PAP-NSM">let him that heareth</w> <w src="12" lemma="strong:G3004 lemma.TR:ειπατω" morph="robinson:V-2AAM-3S">say</w>, <w src="13" lemma="strong:G2064 lemma.TR:ελθε" morph="robinson:V-2AAM-2S">Come</w>. <w src="14" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">And</w> <w src="15 16" lemma="strong:G3588 strong:G1372 lemma.TR:ο lemma.TR:διψων" morph="robinson:T-NSM robinson:V-PAP-NSM">let him that is athirst</w> <w src="17" lemma="strong:G2064 lemma.TR:ελθετω" morph="robinson:V-2AAM-3S">come</w>. <w src="18" lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ">And</w> <w src="19 20" lemma="strong:G3588 strong:G2309 lemma.TR:ο lemma.TR:θελων" morph="robinson:T-NSM robinson:V-PAP-NSM">whosoever will</w>, <w src="21" lemma="strong:G2983 lemma.TR:λαμβανετω" morph="robinson:V-PAM-3S">let him take</w> <w src="22 23" lemma="strong:G3588 strong:G5204 lemma.TR:το lemma.TR:υδωρ" morph="robinson:T-ASN robinson:N-ASN">the water</w> <w src="24" lemma="strong:G2222 lemma.TR:ζωης" morph="robinson:N-GSF">of life</w> <w src="25" lemma="strong:G1432 lemma.TR:δωρεαν" morph="robinson:ADV">freely</w>.<verse eID="Rev.22.17"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Rev.22.17" sID="Rev.22.17"/><w>And</w> <w>the Spirit</w> <w>and</w> <w>the bride</w> <w>say</w>, <w>Come</w>. <w>And</w> <w>let him that heareth</w> <w>say</w>, <w>Come</w>. <w>And</w> <w>let him that is athirst</w> <w>come</w>. <w>And</w> <w>whosoever will</w>, <w>let him take</w> <w>the water</w> <w>of life</w> <w>freely</w>.<verse eID="Rev.22.17"/>
```

### REV.22.20 (OSIS `Rev.22.20`)

- offset 28040931, length 1136, excerpt SHA-256 `7a217b64bc2e762d6fb3969920154798a36d7fca98428c66c9b783ff5f215549`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 1; `</q>` end tags in excerpt: 1

```xml
<verse osisID="Rev.22.20" sID="Rev.22.20"/><w src="2" lemma="strong:G3588 lemma.TR:ο" morph="robinson:T-NSM">He which</w> <w src="3" lemma="strong:G3140 lemma.TR:μαρτυρων" morph="robinson:V-PAP-NSM">testifieth</w> <w src="4" lemma="strong:G3778 lemma.TR:ταυτα" morph="robinson:D-APN">these things</w> <w src="1" lemma="strong:G3004 lemma.TR:λεγει" morph="robinson:V-PAI-3S">saith</w>, <q who="Jesus" marker=""><w src="5" lemma="strong:G3483 lemma.TR:ναι" morph="robinson:PRT">Surely</w> <w src="6" lemma="strong:G2064 lemma.TR:ερχομαι" morph="robinson:V-PNI-1S">I come</w> <w src="7" lemma="strong:G5035 lemma.TR:ταχυ" morph="robinson:ADV">quickly</w>.</q> <w src="8" lemma="strong:G281 lemma.TR:αμην" morph="robinson:HEB">Amen</w>. <w src="9" lemma="strong:G3483 lemma.TR:ναι" morph="robinson:PRT">Even so</w>, <w src="10" lemma="strong:G2064 lemma.TR:ερχου" morph="robinson:V-PNM-2S">come</w>, <w src="11" lemma="strong:G2962 lemma.TR:κυριε" morph="robinson:N-VSM">Lord</w> <w src="12" lemma="strong:G2424 lemma.TR:ιησου" morph="robinson:N-VSM">Jesus</w>.<verse eID="Rev.22.20"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Rev.22.20" sID="Rev.22.20"/><w>He which</w> <w>testifieth</w> <w>these things</w> <w>saith</w>, <q who="Jesus" marker=""><w>Surely</w> <w>I come</w> <w>quickly</w>.</q> <w>Amen</w>. <w>Even so</w>, <w>come</w>, <w>Lord</w> <w>Jesus</w>.<verse eID="Rev.22.20"/>
```

### GEN.1.1 (OSIS `Gen.1.1`)

- offset 4541, length 326, excerpt SHA-256 `7984137f2443b5c260ae118cdcc819490f55f6ec4913d620bcf491ef704239eb`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="Gen.1.1" sID="Gen.1.1"/><w lemma="strong:H07225">In the beginning</w> <w lemma="strong:H0430">God</w> <w morph="strongMorph:TH8804" lemma="strong:H0853 strong:H01254">created</w> <w lemma="strong:H08064">the heaven</w> <w lemma="strong:H0853">and</w> <w lemma="strong:H0776">the earth</w>.<verse eID="Gen.1.1"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Gen.1.1" sID="Gen.1.1"/><w>In the beginning</w> <w>God</w> <w>created</w> <w>the heaven</w> <w>and</w> <w>the earth</w>.<verse eID="Gen.1.1"/>
```

### PSA.23.1 (OSIS `Ps.23.1`)

- offset 9464461, length 309, excerpt SHA-256 `060f4510592a33634c6382ff7c68d41e658b6300353078e22ffb17ba512332d4`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="Ps.23.1" sID="Ps.23.1"/><w lemma="strong:H03068">The <divineName>Lord</divineName></w> <transChange type="added">is</transChange> <w morph="strongMorph:TH8802" lemma="strong:H07462">my shepherd</w>; <w morph="strongMorph:TH8799" lemma="strong:H02637">I shall not want</w>.<verse eID="Ps.23.1"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Ps.23.1" sID="Ps.23.1"/><w>The <divineName>Lord</divineName></w> <transChange type="added">is</transChange> <w>my shepherd</w>; <w>I shall not want</w>.<verse eID="Ps.23.1"/>
```

### PSA.22.1 (OSIS `Ps.22.1`)

- offset 9449355, length 705, excerpt SHA-256 `4fa913997fcaa2ea0adcb449d02e78d8405f368948cf6722f15076e766ec8aa2`
- `q` open at verse start: no
- `q` open at verse end: no
- `<q>` start tags in excerpt: 0; `</q>` end tags in excerpt: 0

```xml
<verse osisID="Ps.22.1" sID="Ps.22.1"/><w lemma="strong:H0410">My God</w>, <w lemma="strong:H0410">my God</w>, <w morph="strongMorph:TH8804" lemma="strong:H05800">why hast thou forsaken</w> me? <transChange type="added">why art thou so</transChange> <w lemma="strong:H07350">far</w> <w lemma="strong:H03444">from helping</w> me, <transChange type="added">and from</transChange> <w lemma="strong:H01697">the words</w> <w lemma="strong:H07581">of my roaring</w>?<note type="study"><catchWord>Aijeleth…</catchWord>: or, <rdg type="alternate">the hind of the morning</rdg></note><note type="study"><catchWord>helping…</catchWord>: Heb. <rdg type="x-literal">my salvation</rdg></note><verse eID="Ps.22.1"/>
```

Reading aid (derived, not authoritative: the block above with the `src`, `lemma`, `morph` and `type` attributes deleted from `<w>` tags; all other bytes unchanged):

```text
<verse osisID="Ps.22.1" sID="Ps.22.1"/><w>My God</w>, <w>my God</w>, <w>why hast thou forsaken</w> me? <transChange type="added">why art thou so</transChange> <w>far</w> <w>from helping</w> me, <transChange type="added">and from</transChange> <w>the words</w> <w>of my roaring</w>?<note type="study"><catchWord>Aijeleth…</catchWord>: or, <rdg type="alternate">the hind of the morning</rdg></note><note type="study"><catchWord>helping…</catchWord>: Heb. <rdg type="x-literal">my salvation</rdg></note><verse eID="Ps.22.1"/>
```
