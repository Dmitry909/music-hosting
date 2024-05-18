import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:http/http.dart' as http;

class SignupPage extends StatefulWidget {
  const SignupPage({super.key});

  @override
  _SignupPageState createState() => _SignupPageState();
}

class _SignupPageState extends State<SignupPage> {
  final TextEditingController _usernameController = TextEditingController();
  final TextEditingController _passwordController = TextEditingController();
  String _signupResult = '';

  Future<void> _signup() async {
    final String username = _usernameController.text;
    final String password = _passwordController.text;

    final Map<String, String> body = {
      'username': username,
      'password': password,
    };

    try {
      final response = await http.post(
        Uri.parse('http://localhost:3000/signup'),
        headers: {'Content-Type': 'application/json'},
        body: jsonEncode(body),
      );

      if (response.statusCode == 201) {
        setState(() {
          _signupResult = 'You were registered';
        });
      } else {
        setState(() {
          _signupResult = 'Signup failed. Please try again.';
        });
      }
    } catch (e) {
      setState(() {
        _signupResult = 'Error occurred. Please try again.';
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Signup Page'),
      ),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            TextField(
              controller: _usernameController,
              decoration: const InputDecoration(labelText: 'Username'),
            ),
            const SizedBox(height: 16),
            TextField(
              controller: _passwordController,
              obscureText: true,
              decoration: const InputDecoration(labelText: 'Password'),
            ),
            const SizedBox(height: 16),
            ElevatedButton(
              onPressed: _signup,
              child: const Text('Sign up'),
            ),
            const SizedBox(height: 16),
            Text(
              _signupResult,
              style: TextStyle(
                color: _signupResult == 'You were registered'
                    ? Colors.green
                    : Colors.red,
              ),
            ),
          ],
        ),
      ),
    );
  }
}
