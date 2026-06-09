%global rust_target x86_64-unknown-linux-gnu
%global debug_package %{nil}

Name:       trex
Version:    0.7.0
Release:    1%{?dist}
Summary:    Tmux Restore Extreme - persist and restore tmux sessions

License:    MIT
URL:        https://github.com/tahasadough/trex
Source0:    %{url}/releases/download/v%{version}/trex-%{rust_target}.tar.gz

Requires:   tmux

%description
Persist your tmux sessions across reboots. Windows, panes, layouts, working
directories, session options, and running commands come back exactly as you
left them.
Single binary, no runtime dependencies beyond tmux.

%prep
%setup -q -c %{name}-%{version}

%install
install -Dm755 %{name} %{buildroot}%{_bindir}/%{name}
install -Dm644 %{name}.1 %{buildroot}%{_mandir}/man1/%{name}.1
install -Dm644 %{SOURCE0} %{buildroot}%{_datadir}/%{name}/license.tar.gz 2>/dev/null || :

%files
%license LICENSE
%{_bindir}/trex
%{_mandir}/man1/trex.1*

%changelog
* Tue Jun 09 2026 Taha Sadough <taha@sadough.dev> - 0.7.0-1
- Initial package release.
