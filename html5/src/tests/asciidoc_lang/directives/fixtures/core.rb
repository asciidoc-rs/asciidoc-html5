# tag::timings[]
if timings
  timings.record :read
  timings.start :parse
end
# end::timings[]
# tag::parse[]
doc = (options[:parse] == false ? (Document.new lines, options) :
    (Document.new lines, options).parse)
timings.record :parse if timings
doc
# end::parse[]
