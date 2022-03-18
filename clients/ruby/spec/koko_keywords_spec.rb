RSpec.describe KokoKeywords do
  it "has a version number" do
    expect(KokoKeywords::VERSION).not_to be nil
  end

  it "works" do
    expect(KokoKeywords.match("sewerslide")).to                               eq(true)
    expect(KokoKeywords.match("sewerslide", filters: "category=wellness")).to eq(false)
    expect(KokoKeywords.match("it's all good")).to                            eq(false)
    expect(KokoKeywords.match("it's all good", version: "20220206")).to       eq(false)
  end
end
