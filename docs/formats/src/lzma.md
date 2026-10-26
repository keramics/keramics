# LZMA compressed data format

LZMA compression is based on the LZ77 sliding window algorithm, in which repeated data is
referenced by a length and a distance, combined with a range coder, which is an entropy coder
that encodes individual bits using probability models.

There are multiple variants of LZMA compressed data:

* LZMA1, also referred to as "plain LZMA" or "LZMA alone"
* LZMA2, which refers to the LZMA variant used inside the [xz compressed data format](xz.md)

Both variants share a common core, such as range coder, state and symbol decoding.

## Overview

LZMA compressed data consist of:

* header
* encoded LZMA stream

### Characteristics

| Characteristics | Description |
| --- | --- |
| Byte order | little-endian |

## Header

The header is 13 bytes in size and consists of:

| Offset | Size | Value | Description |
| --- | --- | --- | --- |
| 0 | 1 | | [Properties](#properties) |
| 1 | 4 | | [Dictionary size](#dictionary_size) |
| 5 | 8 | | [Uncompressed data size](#uncompressed_data_size) |

### Properties {#properties}

The properties byte encodes the number of literal context bits (lc), literal position bits (lp)
and position bits (pb) used by the LZMA model:

| Property | Description |
| --- | --- |
| lc | \[0, 8], number of literal context bits |
| lp | \[0, 4], number of literal position bits |
| pb | \[0, 4], number of position bits |

The properties byte is encoded with:

```python
properties = (
    (((position_bits * 5) + literal_position_bits) * 9) + literal_context_bits
)
```

and decoded with:

```python
if properties >= (9 * 5 * 5)
    error "Incorrect LZMA properties"

literal_context_bits = properties % 9
properties  = properties / 9
position_bits = properties / 5
literal_position_bits = properties % 5
```

Also there is a limitation that (lc + lp) must be less than or equal to 4.

### Dictionary size {#dictionary_size}

The dictionary (or sliding window) size is a 32-bit little-endian value in the range \[0, 2^32 - 1].
If the value is smaller than 4096 (0x1000), the decoder must use a dictionary size of 4096.

### Uncompressed data size {#uncompressed_data_size}

The uncompressed data size is a 64-bit little-endian value.

If the value contains 1s in all 64 bits (0xffffffffffffffff), the uncompressed data size is
unknown and the stream contains an [end of payload marker](#end_of_payload_marker) that indicates
the end of the (compressed) stream.

Otherwise the decoding must be finished after the specified number of bytes has been decoded, and
the [end of payload marker](#end_of_payload_marker) is optional, but if present, it must be read
as well.

## Encoded LZMA stream

The encoded LZMA stream consists of one or more LZMA symbols, each of which is either:

* a [literal](#literal)
* a [match](#match)
* the [end of payload marker](#end_of_payload_marker)

The symbols are encoded with a range coder using probability models.

### Range coder

A range coder maintains a range, initialized to 0xffffffff, and a code value:

```text
Range = 0xffffffff
kTopValue = 1 << 24
```

The range coder is initialized with the first 5 bytes of the stream. The first byte must be 0 and
the 4 remaining bytes are read as a 32-bit big-endian code value.

```text
if first_byte != 0
    error "Invalid range coder initialization"

code  = read 4 bytes, big-endian
range = 0xffffffff
```

During decoding the code value is normalized, to keep it in the same order of magnitude as the
range:

```text
if range < kTopValue
    read next byte
    range <<= 8
    code  = (code << 8) | byte
```

When decoding has properly finished the code value is always 0, unless the input stream is corrupt.
Checking whether the code value is zero after a final normalization, can therefore be used to
detect a well-formed end of the stream.

A bit is decoded using an 11-bit probability model:

```text
kNumBitModelTotal = 1 << 11   /* 2048 */
kNumMoveBits      = 5
PROB_INIT_VAL     = kNumBitModelTotal >> 1

bound = (range >> 11) * probability

if code < bound
    bit   = 0
    range = bound
    prob  += (kNumBitModelTotal - prob) >> kNumMoveBits
else
    bit   = 1
    code  -= bound
    range -= bound
    prob  -= prob >> kNumMoveBits

normalize
```

A bit can also be decoded without using a probability model:

```text
range >>= 1
if code >= range
    bit  = 1
    code -= range
else
    bit  = 0
```

A probability bit tree, used to decode a value in the range \[0, 2^n - 1], decodes n bits one at a
time, using a single probability per bit tree level:

```text
symbol = 1
for i = 0 .. n - 1
    symbol = (symbol << 1) | DecodeBit(&probabilities[symbol])

value = symbol - 2^n
```

A reverse probability bit tree decodes the same value, but with the least significant bit (LSB)
first:

```text
value = 0
for i = 0 .. n - 1
    value += DecodeBit(&probabilities[1 + i]) << i
```

### State

The LZMA decoder uses 12 states to predict which type of LZMA symbol comes next:

* the lowest 7 (0 to 6) indicate that the previous symbol was a literal
* 7 to 11 indicate that the previous symbol was a match

```text
kNumStates    = 12
kNumLitStates = 7
```

Each state has a probability whether a literal or a match comes next. The states are updated
after decoding a symbol:

```text
UpdateState_Literal(state)
    if state < 4
        return 0
    else if state < 10
        return state - 3
    else
        return state - 6

UpdateState_Match(state)
    return state < 7 ? 7 : 10

UpdateState_Rep(state)
    return state < 7 ? 8 : 11

UpdateState_ShortRep(state)
    return state < 7 ? 9 : 11
```

The decoder uses a position state, based on the number of position bits (pb), to select
a probability for some of the models. The position state is the lowest pb bits of the current
output position, as the probability of literal/match values changes depending on the position in
a file:

```text
posState = OutWindow.TotalPos & ((1 << pb) - 1)
```

### Symbol decoding

The decoder uses the following code flow to select the type of LZMA symbol:

```text
IsMatch[state, posState]
  0 - Literal
  1 - Match
     IsRep[state]
       0 - Simple Match ("Simple Match", distance encoded with bit models)
       1 - Rep Match
          IsRep0[state]
            0 - Rep Match 0
               IsRep0Long[state, posState]
                 0 - Short Rep Match (single byte at rep0 distance)
                 1 - Rep Match 0 (long match at rep0 distance)
            1 -
               IsRep1[state]
                 0 - Rep Match 1
                 1 -
                    IsRep2[state]
                      0 - Rep Match 2
                      1 - Rep Match 3
```

The decoder keeps a history of the 4 latest match distances (rep0 to rep3), containing zero-based
distances, initialized to zero. The distance history is updated depending on the type of match:

* Simple Match:
  `rep3 = rep2; rep2 = rep1; rep1 = rep0; rep0 = distance`
* Rep Match 0 or Short Rep Match:
  no update
* Rep Match 1:
  `dist = rep1; rep1 = rep0; rep0 = dist`
* Rep Match 2:
  `dist = rep2; rep2 = rep1; rep1 = rep0; rep0 = dist`
* Rep Match 3:
  `dist = rep3; rep3 = rep2; rep2 = rep1; rep1 = rep0; rep0 = dist`

### Literal

If a value of 0 was decoded with IsMatch, the LZMA symbol is a literal, a single byte value.
The literal decoder must check that decoding the literal does not exceed the specified
uncompressed data size.

There are three types of probabilities for literals:

| Type | Number of values | Description |
| --- | --- | --- |
| Literal coder | 1 byte symbol, 2^8 - 1 | Without match byte |
| Matched literal coder, match bit 0 | 1 byte symbol, 2^8 - 1 | With match byte, match bit is 0 |
| Matched literal coder, match bit 1 | 1 byte symbol, 2^8 - 1 | With match byte, match bit is 1 |

The decoder uses the number of literal context bits (lc) and literal position bits (lp) to select
a probability model for the literal (matched literal) symbols:

```text
literal_state = ((OutWindow.TotalPos & ((1 << lp) - 1)) << lc) + (previous_byte >> (8 - lc))
probabilities = literal_probabilities[literal_state * 0x300]
```

where previous_byte is the last decoded byte, or zero if no byte has been decoded yet.

If the previous symbol was a match, a matched literal coder is used to predict the literal value.
The match byte is the byte at the position of the latest decoded match (rep0), or zero if the
position is not yet available. For each bit of the literal symbol, the decoder first tries to
predict the bit using the match byte, and if the prediction differs from the decoded bit, the
rest of the symbol is decoded with the non-matched literal coder:

```text
symbol = 1
matchByte = the byte at rep0 of the latest match

if previous state was a match
  while symbol < 0x100
    matchBit = (matchByte >> 7) & 1
    matchByte <<= 1

    if DecodeBit(probabilities[((1 + matchBit) * 0x100 + symbol]) != matchBit
      break

    symbol = (symbol << 1) | matchBit

while symbol < 0x100
  symbol = (symbol << 1) | DecodeBit(probabilities[symbol])

put literal byte (symbol & 0xff) in output stream
```

### Match {#match}

If a value of 1 was decoded with IsMatch, the LZMA symbol is a match. A match is defined by a
length and a distance. The decoder first decodes the [match length](#match_length) and then the
distance, or selects one of the 4 distances of the distance history.

The decoder must check that decoding the match does not exceed the specified uncompressed data
size, that the distance is not larger than the dictionary size, and that the distance is not
larger than the number of already decoded bytes.

#### Match length {#match_length}

There are two types of length decoders, one for simple matches and one for repeated matches. Each
length decoder consists of:

* a choice bit, which indicates a low-length or higher length
* a low-length decoder, a 3-bit bit tree (8 values) per position state
* a second choice bit, which indicates a mid-length or high-length
* a mid-length decoder, a 3-bit bit tree (8 values) per position state
* a high-length decoder, an 8-bit bit tree (256 values)

The decoded match length (one-based) is:

| Choice bits | Length bits | Length range |
| --- | --- | --- |
| 0 x | 3 | 2 - 9 |
| 1 0 | 3 | 10 - 17 |
| 1 1 | 8 | 18 - 273 |

The maximum match length is 273.

```text
if DecodeBit(LenChoice) == 0
  return 2 + LowLength[posState]
if DecodeBit(LenChoice2) == 0
  return 10 + MidLength[posState]

return 18 + HighLength
```

#### Match distance {#match_distance}

The distance decoder uses the normalized zero-based match length to select a state from which it
selects a probability model for the distance:

```text
kNumLenToPosStates = 4

lenState = LEN if LEN < 4 else 3 (where LEN is the match length minus 2)
```

The distance decoder decodes a slot (6-bit bit tree per state) and based on the slot, either the
distance value itself, the lowest distance bits using a shared probability model, or a
combination of plain bits and an alignment probability model:

```text
kEndPosModelIndex  = 14
kNumAlignBits      = 4
kNumFullDistances  = 1 << (kEndPosModelIndex >> 1)  /* 128 */
```

The number of distance slot (and therefore the range of distances) that a slot covers:

| Slot | Distance range | Distance bits |
| --- | --- | --- |
| 0 | 0 | 0 |
| 1 | 1 | 0 |
| 2 | 2 | 0 |
| 3 | 3 | 0 |
| 4 | 4 - 5 | 1 |
| 5 | 6 - 7 | 1 |
| 6 | 8 - 11 | 2 |
| 7 | 12 - 15 | 2 |
| 8 | 16 - 23 | 3 |
| 9 | 24 - 31 | 3 |
| 10 | 32 - 47 | 4 |
| 11 | 48 - 63 | 4 |
| 12 | 64 - 95 | 5 |
| 13 | 96 - 127 | 5 |
| 14 | 128 - 255 | 6 + 4 |
| 15 | 256 - 511 | 7 + 4 |
| ... | ... | n + 4 |
| 63 | 0x20000000 - 0xffffffff | 29 + 4 |

The decoding is:

```text
posSlot = PosSlot[lenState].Decode()

if posSlot < 4
  return posSlot

numDirectBits = (posSlot >> 1) - 1
dist          = (2 | (posSlot & 1)) << numDirectBits

if posSlot < kEndPosModelIndex
  dist += BitTreeReverseDecode(PosDecoders + dist - posSlot, numDirectBits)
else
  dist += DecodeDirectBits(numDirectBits - kNumAlignBits) << kNumAlignBits
  dist += Align.ReverseDecode()

return dist
```

The distance is returned as a zero-based value, i.e. the number of bytes to move backwards in the
output stream before copying the match byte sequence.

### Repeated match

A repeated match reuses one of the 4 distances of the distance history (rep0 to rep3). The decoder
must check that the sliding window is not empty (i.e. at least one byte has been decoded) and
perform the same distance checks as for a [match](#match).

The length of a repeated match is decoded with the repeated match length decoder and the match bytes
are copied from the output stream, same as for a simple match.

A short repeated match is a repeated match of length 1 at the rep0 distance: a single byte which is
the byte at rep0 of the already decoded output stream.

### End of payload marker {#end_of_payload_marker}

The match distance value 0xffffffff (all 32 bits set) is reserved for the end of payload marker
(EOPM). The EOPM is encoded as a simple match with a distance of 0xffffffff (posSlot 63, with
all direct bits and alignment bits set).

The decoder must stop decoding when the EOPM is detected and check that the range coder is
finished (i.e. after a final normalization the code value is zero):

```text
if rep0 == 0xffffffff
    return RangeDec.IsFinishedOK() ? FINISHED_WITH_MARKER : ERROR
```

The EOPM can only appear at the end of the stream: when the uncompressed data size is undefined,
the EOPM is mandatory, and when the uncompressed data size is defined, the EOPM may only appear
after the specified number of bytes has been decoded.

## Decoding

The main loop of the LZMA decoder is as described below.

```text
loop
{
    if the uncompressed data size is defined and has been reached
      if the range coder is finished (code value is zero)
        return FINISHED_WITHOUT_MARKER

    decode IsMatch[state, posState]

    if symbol is a literal
      check end of stream conditions
      decode and put the literal in the output stream
      if no more output space is available and the stream is finished
        return FINISHED_WITH_MARKER / FINISHED_WITHOUT_MARKER
      state = UpdateState_Literal(state)
      continue

    if symbol is a match
      check end of stream conditions

      if a simple match
        state = UpdateState_Match(state)
        decode the match length
        rep0 = DecodeDistance(len)

        if rep0 == 0xffffffff
          return RangeDec.IsFinishedOK() ? FINISHED_WITH_MARKER : ERROR

        update the distance history with rep0
        check that rep0 is within the dictionary size and already decoded bytes

      if a repeated match
        check that the sliding window is not empty
        select the distance (rep0 to rep3) from the distance history and update
        the distance history
        state = UpdateState_Rep(state) (or UpdateState_ShortRep for a short repeated match)
        decode the match length (skip for a short repeated match)

      copy the match byte sequence from the sliding window to the current
      position in the output stream
}
```

## References

* [LZMA specification (DRAFT version)](https://github.com/jljusten/LZMA-SDK/blob/master/DOC/lzma-specification.txt)
* [7-Zip format](https://github.com/jljusten/LZMA-SDK/blob/master/DOC/7zFormat.txt)
