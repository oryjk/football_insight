import 'package:firebase_core/firebase_core.dart';
import 'package:flutter/material.dart';
import 'package:football_insight_app/app.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  try {
    await Firebase.initializeApp();
  } catch (_) {}
  runApp(const ProviderScope(child: FootballInsightApp()));
}
