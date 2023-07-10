// File generated by FlutterFire CLI.
// ignore_for_file: lines_longer_than_80_chars
import 'package:firebase_core/firebase_core.dart' show FirebaseOptions;
import 'package:flutter/foundation.dart'
    show defaultTargetPlatform, kIsWeb, TargetPlatform;

/// Default [FirebaseOptions] for use with your Firebase apps.
///
/// Example:
/// ```dart
/// import 'firebase_options.dart';
/// // ...
/// await Firebase.initializeApp(
///   options: DefaultFirebaseOptions.currentPlatform,
/// );
/// ```
class DefaultFirebaseOptions {
  static FirebaseOptions get currentPlatform {
    if (kIsWeb) {
      return web;
    }
    // ignore: missing_enum_constant_in_switch
    switch (defaultTargetPlatform) {
      case TargetPlatform.android:
        return android;
      case TargetPlatform.iOS:
        return ios;
      case TargetPlatform.macOS:
        return macos;
      case TargetPlatform.fuchsia:
      case TargetPlatform.linux:
      case TargetPlatform.windows:
        break;
    }

    throw UnsupportedError(
      'DefaultFirebaseOptions are not supported for this platform.',
    );
  }

  static const FirebaseOptions web = FirebaseOptions(
    apiKey: 'AIzaSyAKM9dg-DKJin0KofrD1VJ4OhoJ4wVK_nM',
    appId: '1:1068438521597:web:06fd7a8f62085fdae12094',
    messagingSenderId: '1068438521597',
    projectId: 'simple-metronome-eb182',
    authDomain: 'simple-metronome-eb182.firebaseapp.com',
    storageBucket: 'simple-metronome-eb182.appspot.com',
    measurementId: 'G-RLTXVYQJB4',
  );

  static const FirebaseOptions android = FirebaseOptions(
    apiKey: 'AIzaSyCwyJGKGJjyijDu-_dXx7EdPMuM_Zun0BE',
    appId: '1:1068438521597:android:bfdc925efa7f13c7e12094',
    messagingSenderId: '1068438521597',
    projectId: 'simple-metronome-eb182',
    storageBucket: 'simple-metronome-eb182.appspot.com',
  );

  static const FirebaseOptions ios = FirebaseOptions(
    apiKey: 'AIzaSyC0n_E8qtlDoATgtmZ_kr_Yy5_3IBJE-Fk',
    appId: '1:1068438521597:ios:9f57977394bd71f5e12094',
    messagingSenderId: '1068438521597',
    projectId: 'simple-metronome-eb182',
    storageBucket: 'simple-metronome-eb182.appspot.com',
    iosClientId:
        '1068438521597-9gpffcgl4r0iin7cohli2s674cn5o624.apps.googleusercontent.com',
    iosBundleId: 'com.beijaflor.metronome',
  );

  static const FirebaseOptions macos = FirebaseOptions(
    apiKey: 'AIzaSyC0n_E8qtlDoATgtmZ_kr_Yy5_3IBJE-Fk',
    appId: '1:1068438521597:ios:9f57977394bd71f5e12094',
    messagingSenderId: '1068438521597',
    projectId: 'simple-metronome-eb182',
    storageBucket: 'simple-metronome-eb182.appspot.com',
    iosClientId:
        '1068438521597-9gpffcgl4r0iin7cohli2s674cn5o624.apps.googleusercontent.com',
    iosBundleId: 'com.beijaflor.metronome',
  );
}
