import unittest
from koko_keywords import match

class TestKokoKeywords(unittest.TestCase):
  def test_match(self):
      self.assertEqual(match("sewerslide"), True)

  def test_non_match_with_filter(self):
      self.assertEqual(match("sewerslide", filters="category=wellness"), False)

  def test_non_match(self):
      self.assertEqual(match("it's all good"), False)

  def test_non_match_with_version(self):
      self.assertEqual(match("it's all good", version="20220206"), False)

  def test_non_match_again_to_test_cache(self):
      self.assertEqual(match("it's all good"), False)

if __name__ == '__main__':
    unittest.main()
