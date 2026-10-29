Pod::Spec.new do |s|
  s.name = 'nanocode'
  s.version = '0.2.1'
  s.summary = 'NanoCode native FFI'
  s.description = 'Experimental circular visual code generator and reader.'
  s.homepage = 'https://github.com/NikGariel/nanocode'
  s.license = { :type => 'MIT', :file => '../LICENSE' }
  s.author = { 'NanoCode contributors' => 'https://github.com/NikGariel/nanocode' }
  s.source = { :path => '.' }
  s.osx.deployment_target = '11.0'
  s.vendored_frameworks = 'CNanoCode.xcframework'
  s.user_target_xcconfig = { 'OTHER_LDFLAGS' => '$(inherited) -Wl,-u,_nc_svg_ex -Wl,-u,_nc_scan_ex -Wl,-u,_nc_palette_score -Wl,-u,_nc_backgrounds' }
end
