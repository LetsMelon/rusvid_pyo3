# rusvid_pyo3

Create simple png files with the help of [`rusvid`](https://github.com/LetsMelon/rusvid). A library written in Rust, called from Python code.

Python code:

```python
from python_ffi import CustomImage

image = CustomImage("./input.raw")
image.save("./out.png")
```

And run with `maturin develop && python FILENAME.py`

<details>
  <summary><code>./input.raw</code></summary>

  ```txt
  width   100
  height  100
  background [255, 250, 100, 75]

  pixel   (0, 0)   [255, 255, 0, 0]
  rect    (10, 10) (50, 50) [255, 20, 50, 100]
  ```
</details>

---

## Dependencies
- Rust nightly
- Python >= 3.7
- [maturin](https://github.com/PyO3/maturin)
