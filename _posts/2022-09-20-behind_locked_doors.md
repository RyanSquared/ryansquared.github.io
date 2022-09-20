---
layout: post
title: Behind Locked Doors
tags: qubes, gnupg, openpgp, ssh, webauthn, security
date: 2022-09-20 19:41 -0400
---
**Foreword:** This blog post will likely change as time goes on and I learn
more about Qubes and how I can adapt my home-built SSH solution to be on par
with the provided SSH solution. I hope to keep the structure of the post
mostly consistent.

Recently, I've started using an operating system at work called [QubesOS]. The
operating system is pretty fancy, it gives me a way to separate all of my
applications into little environment-specific containers. It's very useful as a
contractor, as I can have many clients at once. With the default 4 workspaces
that Qubes provides me, that gives me the ability to move between my personal
Qube, my Distrust Qube, and a couple Qubes I have set up for clients. It offers
me an excellent way to make sure that both the visual component - the programs
that I'm using, whether it be Firefox, Thunderbird, or a terminal - and the
data components, such as the actual project files themselves, are all
self-contained.

However, each of these clients, in one way or another, needs me to be able to
interact with them cryptographically in a way that I can trust. This can be
unlocking a password database, signing a GitHub commit, connecting to a server
via SSH, or logging into a service that incorporates a WebAuthn authentication
flow. I need a way to be able to take all four of these qubes and hook them
into a central point of authority that I'm able to trust with access to
cryptographic operations involving my private key.

# The Vault

QubesOS by default comes with a qube called the "vault". This is a component
whose compromise would come second only to dom0. It is the centralized
authority for all of my sensitive cryptographic operations. If I want to do
anything that requires access to PGP, SSH, or WebAuthn, it would be run through
this component first. For my personal system, it's based off the Debian 11
template, so while most of these commands should have a Fedora alternative (or
otherwise, if Qubes starts offering new TemplateVMs), the instructions I give
are targeted towards a Debian system.

The first thing I wanted to do was to make sure that my smartcard could
properly attach to the Vault. I used the [sys-usb] setup to connect my
smartcard, a [Yubikey], to the qube. I was able to confirm that the device was
visible from inside the qube by running `lsusb`. From there, since `gpg2` and
`gpg-agent` were installed by default, I was able to run `gpg --card-status` to
confirm that the 

**Note:** At this point, if the `gpg --card-status` command fails, you should
stop what you're doing and make sure that `gpg2` is installed in your vault
qube and that the `gpg-agent` systemd user service has been started.

The Qubes team has created a tool called [qubes-gpg-client] and
`qubes-gpg-client-wrapper` which allows an almost seamless replacement for
`gpg` and `gpg2` for programs such as Git, Thunderbird, and otherwise. It is
close enough to the point where I personally symlink `/usr/local/bin/gpg{,2}`
to `/usr/bin/qubes-gpg-client-wrapper`. I also make sure to put `export
QUBES_GPG_DOMAIN=vault` in my `.zshrc` file, to ensure any program I run will
know which qube to connect to. This can be tested by running
`QUBES_GPG_DOMAIN=vault qubes-gpg-client-wrapper --list-keys`, which will
prompt me for access before performing the operation, which is incredible when
compared to alternative solutions. Be sure to run `qubes-gpg-import-key` to add
your GPG key to the system, and run `qubes-gpg-client-wrapper -K` to ensure you
have access to the private key.

While the Qubes team offers an out-of-the-box option for GnuPG and OpenPGP
operations, their support for SSH is less than ideal. For SSH operations, I
will be working off an [adapted version] of the [community Split SSH] guide.
While the setup for the GPG configuration didn't involve much tinkering with
the system, we do need to run some commands for `dom0`, `vault`, and the
TemplateVM that I plan on using for other qubes.

### dom0

Run the following command to give access to all VMs to use SSH.  You can use
`ask,default_target=vault` instead of `allow` if you'd like a popup before the
SSH operation is allowed to continue.

```sh
echo "@anyvm vault allow" | sudo tee /etc/qubes-rpc/policy/qubes.Ssh
```

### Vault's TemplateVM

Place the following in `/etc/qubes-rpc/qubes.Ssh`:

```sh
#!/bin/sh

export SSH_AUTH_SOCK=/run/user/1000/gnupg/S.gpg-agent.ssh
notify-send "[$(qubesdb-read /name)] SSH access from: $QREXEC_REMOTE_DOMAIN"
socat - "UNIX-CONNECT:$SSH_AUTH_SOCK"
```

And enable the executable flag:

```sh
chmod +x /etc/qubes-rpc/qubes.Ssh
```

### AppVM's TemplateVM

**Note:** This section could probably ~~be trivialized with the use of
generics~~ exist outside of `/rw/config/rc.local` but the original guide used
that location. It could also make use of a similar "allow for timeframe" system
that the gpg vault does, but it was usable so I just moved on.

Note that if you have something in this file already, it should be merged in
with this configuration manually, not completely replaced like I'm doing here.
This is mostly designed as "self documentation" for whenever I set up a new
Qubes system.

Place the following in `/rw/config/rc.local`:

```sh
#!/bin/sh

# This script will be executed at every VM startup, you can place your own
# custom commands here. This includes overriding some configuration in /etc,
# starting services etc.

# Example for overriding the whole CUPS configuration:
#  rm -rf /etc/cups
#  ln -s /rw/config/cups /etc/cups
#  systemctl --no-block restart cups
SSH_VAULT_VM=vault
USER_ID=1000
USER_NAME=`id -nu $USER_ID`
SSH_AUTH_SOCK="/home/$USER_NAME/.ssh/S.ssh-agent"

if [ ! "$SSH_VAULT_VM" = "" ]; then
  sudo -u "$USER_NAME" mkdir -p "$(dirname $SSH_AUTH_SOCK)" 2>/tmp/output
  rm -f "$SSH_AUTH_SOCK"
  sudo -u "$USER_NAME" /bin/sh -c "umask 177 && exec socat 'UNIX-LISTEN:$SSH_AUTH_SOCK,fork' 'EXEC:qrexec-client-vm $SSH_VAULT_VM qubes.Ssh'" &
fi
```

And this in your shell's rc or env file:

```sh
export SSH_AUTH_SOCK="$HOME/.ssh/S.gpg-agent"
```

After restarting your TemplateVMs, Vault, and the AppVM you're using for
testing, you should be able to run `ssh-add -L` and get a notification popup.

<!-- TODO: this section isn't the best. Please add more information about U2F
and WebAuthn, and what they do to protect your privacy compared to TOTP and SMS
2FA. -->

I also followed the instructions for the [Qubes U2F Proxy] to the letter to
get a WebAuthn workflow working to the same device I have attached for PGP and
SSH and despite being slightly slow at times, for the most part it works fine.
I'm able to test this by going to [https://webauthn.io](https://webauthn.io).
WebAuthn is a protocol created by the [FIDO Alliance] to create an origin-based
signature system that, when run in a trusted environment such as a browser or
desktop session, can't be spoofed. This is due to the fact each cryptographic
signature is tied to an origin. Therefore, if I have a WebAuthn setup
configured for `github.com`, `guthib.com` can't spoof it because they'd *also*
make the device believe that it's requesting a signature for `github.com`.

A friend of mine recently [blogged about 2FA methods](xe-push-2fa-c-h). It's an
interesting read that I think gives a good oversight into how the entire
security ecosystem relies on the possibility of human mistake. SMS and TOTP 2FA
are phishable by not having *anything* tied to the domain name, while
notification 2FA causes notification fatigue while also not being phishing
resistant as there's no way to verify the request is coming from an authentic
source - while the issue mentioned in the aforementioned post could have been
due to absentmindedness, there's no inherent way of verifying the origin of the
request.

**Note:** I have had minor issues with CloudFlare which I haven't been able to
debug yet, so if anyone runs into similar issues with CloudFlare and WebAuthn
not working in Qubes U2F Proxy, I'd be happy to either hear a solution or at
least hear that it's not just me.

[QubesOS]: https://qubes-os.org/
[Yubikey]: https://www.yubico.com/products/yubikey-5-overview/
[sys-usb]: https://www.qubes-os.org/doc/usb-qubes/
[qubes-gpg-client]: https://www.qubes-os.org/doc/split-gpg/
[adapted version]: https://github.com/hashbang/book/blob/master/content/docs/security/qubes/vault.md#creating-a-socket-for-ssh
[community Split SSH]: https://github.com/Qubes-Community/Contents/blob/master/docs/configuration/split-ssh.md
[Qubes U2F Proxy]: https://www.qubes-os.org/doc/u2f-proxy/
[FIDO Alliance]: https://fidoalliance.org/
[xe-push-2fa-c-h]: https://xeiaso.net/blog/push-2fa-considered-harmful
